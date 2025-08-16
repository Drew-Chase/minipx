use crate::config::Config;
use anyhow::{Result, anyhow};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode, header};
use log::{error, info, warn};
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};

pub async fn start_rp_server() -> Result<()> {
    loop {
        let addr = SocketAddr::from(([0, 0, 0, 0], 80));

        let make_svc = make_service_fn(move |conn: &AddrStream| {
            let remote_addr = conn.remote_addr().ip();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let client_ip = remote_addr;
                    async move {
                        match handle_request_with_scheme(client_ip, req, false).await {
                            Ok(resp) => Ok::<_, Infallible>(resp),
                            Err(e) => {
                                error!("handle_request error from {}: {}", client_ip, e);
                                Ok::<_, Infallible>(
                                    Response::builder()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .body(Body::empty())
                                        .unwrap(),
                                )
                            }
                        }
                    }
                }))
            }
        });

        let builder = match hyper::Server::try_bind(&addr) {
            Ok(b) => b,
            Err(e) => {
                error!("Failed to bind reverse proxy on {}: {}", addr, e);
                // No config port to wait for; sleep and retry
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                continue;
            }
        };

        let server = builder.serve(make_svc);

        info!("Reverse Proxy Server running on {}", addr);

        if let Err(e) = server.await {
            error!("Server error: {}", e);
            // Loop will retry bind/start
        }
    }
}

fn extract_host(req: &Request<Body>) -> Option<String> {
    if let Some(authority) = req.uri().authority() {
        return Some(authority.host().to_string());
    }
    if let Some(hv) = req.headers().get(header::HOST) {
        if let Ok(host) = hv.to_str() {
            let host_only = host.split(':').next().unwrap_or(host);
            return Some(host_only.to_string());
        }
    }
    req.uri().host().map(|h| h.to_string())
}

pub async fn handle_request_with_scheme(client_ip: IpAddr, req: Request<Body>, is_tls: bool) -> Result<Response<Body>> {
    let uri = req.uri().clone();
    let domain = extract_host(&req).ok_or(anyhow!("No host in URI or Host header"))?;

    let config = Config::get().await;
    let route = config.lookup_host(&domain);

    if route.is_none() {
        warn!("Received request from {ip} for unknown host {host}", ip = client_ip, host = domain);
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(Body::from("Not Found"))?);
    }

    let route = route.unwrap();

    // If this is an HTTPS request but the route is configured for HTTP (no SSL), redirect to HTTP on default port 80.
    if is_tls && !route.is_ssl_enabled() {
        let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        let location = format!("http://{}{}", domain, path_and_query);
        return Ok(
            Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, location)
                .body(Body::empty())?
        );
    }

    // If this is an HTTP request and the route requires HTTPS, redirect only if TLS can be served for this host.
    if !is_tls && route.get_redirect_to_https() {
        if config.can_serve_tls_for_host(&domain) {
            let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
            let location = format!("https://{}{}", domain, path_and_query);
            return Ok(
                Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header(header::LOCATION, location)
                    .body(Body::empty())?
            );
        } else {
            warn!("HTTPS redirect requested for host '{}' but TLS is unavailable (ssl disabled, invalid email, or invalid domain). Serving over HTTP.", domain);
        }
    }

    let target = route.get_full_url();
    info!(
        "Received request from {ip} for {host} redirecting to {route}{path}",
        ip = client_ip,
        path = uri,
        host = domain,
        route = target
    );
    match hyper_reverse_proxy::call(client_ip, target.as_str(), req).await {
        Ok(response) => Ok(response),
        Err(_error) => Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty())?),
    }
}
