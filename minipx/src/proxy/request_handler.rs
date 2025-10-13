use crate::config::Config;
use crate::config::types::ProxyPathRoute;
use crate::proxy::websocket::{is_websocket, proxy_websocket};
use anyhow::{Result, anyhow};
use hyper::{Body, Request, Response, StatusCode, header};
use log::{debug, error, info, warn};
use std::net::IpAddr;

/// Extract the host from the request URI or Host header
pub fn extract_host(req: &Request<Body>) -> Option<String> {
	if let Some(authority) = req.uri().authority() {
		return Some(authority.host().to_string());
	}

	#[allow(clippy::collapsible_if)]
	if let Some(hv) = req.headers().get(header::HOST) {
		if let Ok(host) = hv.to_str() {
			let host_only = host.split(':').next().unwrap_or(host);
			return Some(host_only.to_string());
		}
	}
	req.uri().host().map(|h| h.to_string())
}

/// Handle HTTP/HTTPS request with the specified frontend scheme
pub async fn handle_request_with_scheme(frontend_scheme: &str, client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>> {
	let mut req = req;
	let uri = req.uri().clone();
	let domain = extract_host(&req).ok_or(anyhow!("No host in URI or Host header"))?;

	let config = Config::get().await;
	let route = config.lookup_host(&domain);

	if route.is_none() {
		warn!("Received request from {ip} for unknown host {host}", ip = client_ip, host = domain);
		return Ok(Response::builder().status(StatusCode::NOT_FOUND).header("Content-Type", "text/plain").body(Body::from("Not Found"))?);
	}

	let route = route.unwrap();

	// If the client sent HTTP and the route requires HTTPS,
	// redirect only if TLS can be served for this host.
	if frontend_scheme.eq_ignore_ascii_case("http") && route.get_redirect_to_https() {
		if config.can_serve_tls_for_host(&domain) {
			let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
			let location = format!("https://{}{}", domain, path_and_query);
			return Ok(Response::builder().status(StatusCode::MOVED_PERMANENTLY).header(header::LOCATION, location).body(Body::empty())?);
		} else {
			warn!(
                "HTTPS redirect requested for host '{}' but TLS is unavailable (ssl disabled, invalid email, or invalid domain). Serving over HTTP.",
                domain
            );
		}
	}

	// Determine upstream scheme based on request type and frontend scheme.
	let upstream_scheme = {
		if is_websocket(&req) {
			// WebSocket upstream uses plain ws to backend; TLS is terminated at the proxy
			"ws"
		} else {
			// Always proxy normal HTTP(S) requests to http upstream per requirement
			"http"
		}
	};

	// Check for matching subroute based on request path
	let sub_route: Option<ProxyPathRoute> =
		route.subroutes.iter().find(|r| r.path != "/" && !r.path.is_empty() && uri.path().starts_with(r.path.as_str())).cloned();
	
	let target = if let Some(sub) = &sub_route {
		// For non-WebSocket requests, rewrite the request URI to strip the subroute base path
		if !is_websocket(&req) {
			debug!("Original Route: {req:?}", req = req);
			let stripped_path = uri.path().strip_prefix(sub.path.as_str()).unwrap_or("/");
			let queries = uri.path_and_query().and_then(|pq| pq.query()).map(|q| format!("?{}", q)).unwrap_or_default();
			let stripped_path = format!("{stripped_path}{queries}");

			// Build new request with modified URI
			let og_headers = req.headers().clone();
			let mut new_req = Request::builder()
				.method(req.method())
				.uri(stripped_path)
				.version(req.version())
				.body(req.into_body())?;
			new_req.headers_mut().clone_from(&og_headers);

			req = new_req;

			debug!("Route after path rewrite: {req:?}", req = req);
		} else {
			debug!("WebSocket request - keeping original URI: {req:?}", req = req);
		}
		format!("{protocol}://{domain}:{port}", protocol = upstream_scheme, domain = route.get_host(), port = sub.port)
	} else {
		debug!("Original Route: {req:?}", req = req);
		format!("{}://{}:{}", upstream_scheme, route.get_host(), route.get_port())
	};

	info!(
        "Received request from {ip} for {fs}://{host}{path} -> {route}{path}",
        fs = frontend_scheme,
        ip = client_ip,
        host = domain,
        route = target,
        path = uri.path()
    );
	debug!("Request details: {req:?}", req = req);

	if is_websocket(&req) {
		debug!("WebSocket upgrade detected: frontend={fs}, upstream={up}", fs = frontend_scheme, up = target);
		let (ws_host, ws_port) = if let Some(sub) = sub_route.clone() {
			(route.get_host(), sub.port)
		} else {
			(route.get_host(), route.get_port())
		};
		
		let subroute_path = sub_route.map(|s| s.path).unwrap_or_default();
		return proxy_websocket(client_ip, req, upstream_scheme, ws_host, ws_port, &subroute_path, &domain).await;
	}

	match hyper_reverse_proxy::call(client_ip, target.as_str(), req).await {
		Ok(response) => Ok(response),
		Err(error) => {
			error!("HTTP proxy error for {host} -> {target}: {err:?}", host = domain, target = target, err = error);
			Ok(Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.header("Content-Type", "text/plain")
				.body(Body::from("Internal Server Error"))?)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hyper::{Request, Body};

	#[test]
	fn test_extract_host_from_uri_authority() {
		let req = Request::builder()
			.uri("http://example.com/path")
			.body(Body::empty())
			.unwrap();

		let host = extract_host(&req);
		assert_eq!(host, Some("example.com".to_string()));
	}

	#[test]
	fn test_extract_host_from_header() {
		let req = Request::builder()
			.uri("/path")
			.header("Host", "api.example.com")
			.body(Body::empty())
			.unwrap();

		let host = extract_host(&req);
		assert_eq!(host, Some("api.example.com".to_string()));
	}

	#[test]
	fn test_extract_host_from_header_with_port() {
		let req = Request::builder()
			.uri("/path")
			.header("Host", "api.example.com:8080")
			.body(Body::empty())
			.unwrap();

		let host = extract_host(&req);
		assert_eq!(host, Some("api.example.com".to_string()));
	}

	#[test]
	fn test_extract_host_priority_uri_over_header() {
		let req = Request::builder()
			.uri("http://uri.example.com/path")
			.header("Host", "header.example.com")
			.body(Body::empty())
			.unwrap();

		// URI authority takes precedence
		let host = extract_host(&req);
		assert_eq!(host, Some("uri.example.com".to_string()));
	}

	#[test]
	fn test_extract_host_none() {
		let req = Request::builder()
			.uri("/path")
			.body(Body::empty())
			.unwrap();

		let host = extract_host(&req);
		assert_eq!(host, None);
	}
}
