use anyhow::Result;
use hyper::Client;
use hyper::body::to_bytes;
use hyper::http::Version;
use hyper::upgrade;
use hyper::{Body, Request, Response, StatusCode, header};
use hyper_tls::HttpsConnector;
use log::{debug, error, warn};
use std::net::IpAddr;
use std::time::Instant;

/// Check if the request is a WebSocket upgrade request
pub fn is_websocket(req: &Request<Body>) -> bool {
    let has_upgrade_ws =
        req.headers().get(header::UPGRADE).and_then(|v| v.to_str().ok()).map(|v| v.eq_ignore_ascii_case("websocket")).unwrap_or(false);
    let has_connection_upgrade =
        req.headers().get(header::CONNECTION).and_then(|v| v.to_str().ok()).map(|v| v.to_ascii_lowercase().contains("upgrade")).unwrap_or(false);
    has_upgrade_ws && has_connection_upgrade
}

/// Handle WebSocket proxy requests with upgrade and bidirectional tunneling
pub async fn proxy_websocket(
    client_ip: IpAddr,
    req: Request<Body>,
    upstream_scheme: &str,
    upstream_host: &str,
    upstream_port: u16,
    subroute_path: &str,
    domain: &str,
) -> Result<Response<Body>> {
    // Build upstream URI: strip subroute path if present, then add requested path_and_query
    let suffix = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    let upstream_path =
        if !subroute_path.is_empty() && suffix.starts_with(subroute_path) { suffix.strip_prefix(subroute_path).unwrap_or("/") } else { suffix };

    // For WebSocket upgrades, always use http:// for upstream connections
    // TLS is terminated at the proxy, so backend connections are plain HTTP
    let upstream_uri = format!("http://{}:{}{}", upstream_host, upstream_port, upstream_path);

    // Extract the request body before moving req to preserve it for both upstream and client upgrade
    let (req_parts, req_body) = req.into_parts();
    let req = Request::from_parts(req_parts, req_body);

    // Prepare a WebSocket handshake request to upstream (force HTTP/1.1)
    let mut builder = Request::builder().method(req.method()).version(Version::HTTP_11).uri(&upstream_uri);

    // Copy headers, but fix Host and X-Forwarded-For
    {
        let headers = req.headers();
        for (name, value) in headers.iter() {
            if name == header::HOST {
                continue;
            }
            // Keep Upgrade/Connection and WS headers intact
            builder = builder.header(name, value);
        }
        let host_header = format!("{}:{}", upstream_host, upstream_port);
        builder = builder.header(header::HOST, host_header);

        // X-Forwarded-For
        const XFF: &str = "x-forwarded-for";
        if let Some(existing) = headers.get(XFF) {
            if let Ok(existing_str) = existing.to_str() {
                let appended = format!("{}, {}", existing_str, client_ip);
                builder = builder.header(XFF, appended);
            }
        } else {
            builder = builder.header(XFF, client_ip.to_string());
        }

        // Log key incoming WS headers for diagnostics
        let h = |n: &str| headers.get(n).and_then(|v| v.to_str().ok()).unwrap_or("-");
        debug!(
            "WS incoming headers: Host={}:{} Origin={} Connection={} Upgrade={} Sec-WebSocket-Key={} Version={} Protocol={} Extensions={}",
            upstream_host,
            upstream_port,
            h("origin"),
            h("connection"),
            h("upgrade"),
            h("sec-websocket-key"),
            h("sec-websocket-version"),
            h("sec-websocket-protocol"),
            h("sec-websocket-extensions"),
        );
    }

    // Use empty body for upstream WebSocket handshake (body not needed for upgrade)
    let upstream_req = builder.body(Body::empty())?;

    // HTTP/1.1 only client for WebSocket upgrades (no HTTP/2 adaptive window)
    // WebSocket upgrades require HTTP/1.1, HTTP/2 causes handshake failures
    let https = HttpsConnector::new();
    let client: Client<_, Body> = Client::builder().build::<_, Body>(https);

    debug!(
        "WS upstream request: {method} {uri} (from {client_ip} for {domain})",
        method = upstream_req.method(),
        uri = &upstream_uri,
        client_ip = client_ip,
        domain = domain
    );

    let start = Instant::now();
    match client.request(upstream_req).await {
        Ok(mut upstream_res) => {
            let elapsed = start.elapsed();
            let status = upstream_res.status();
            debug!(
                "WS upstream responded for {domain} -> {uri} with {status} in {ms} ms",
                domain = domain,
                uri = upstream_uri,
                status = status,
                ms = elapsed.as_millis()
            );
            if status != StatusCode::SWITCHING_PROTOCOLS {
                // Collect headers for diagnostics
                let mut hdrs = String::new();
                for (k, v) in upstream_res.headers().iter() {
                    if let Ok(vs) = v.to_str() {
                        hdrs.push_str(&format!("{}: {}; ", k.as_str(), vs));
                    }
                }
                // Capture body (truncate for logging)
                let body_bytes = to_bytes(upstream_res.body_mut()).await.unwrap_or_default();
                let mut body_preview = String::from_utf8_lossy(&body_bytes).to_string();
                if body_preview.len() > 2048 {
                    body_preview.truncate(2048);
                    body_preview.push('…');
                }
                warn!(
                    "WS upstream non-101 for {domain} -> {uri}: {status}; headers=<{hdrs}> body[preview]={preview}",
                    domain = domain,
                    uri = upstream_uri,
                    status = status,
                    hdrs = hdrs,
                    preview = body_preview
                );
                // Rebuild response to the client with same status/headers/body
                let mut resp_builder = Response::builder().status(status);
                for (k, v) in upstream_res.headers().iter() {
                    resp_builder = resp_builder.header(k, v.clone());
                }
                return Ok(resp_builder.body(Body::from(body_bytes))?);
            }

            // Prepare 101 response to the client, mirroring key headers from upstream
            let mut resp_builder = Response::builder().status(StatusCode::SWITCHING_PROTOCOLS);
            for &h in [
                &header::UPGRADE,
                &header::CONNECTION,
                &header::SEC_WEBSOCKET_ACCEPT,
                &header::SEC_WEBSOCKET_PROTOCOL,
                &header::SEC_WEBSOCKET_EXTENSIONS,
            ]
            .iter()
            {
                if let Some(v) = upstream_res.headers().get(h) {
                    resp_builder = resp_builder.header(h, v.clone());
                }
            }
            let response_to_client = resp_builder.body(Body::empty())?;

            // Spawn tunnel task to bridge upgraded connections
            let domain_owned = domain.to_string();
            let uri_owned = upstream_uri.clone();
            tokio::spawn(async move {
                // Wait for client upgrade
                match upgrade::on(req).await {
                    Ok(mut upgraded_client) => {
                        // Wait for upstream upgrade
                        match upgrade::on(upstream_res).await {
                            Ok(mut upgraded_upstream) => {
                                if let Err(e) = tokio::io::copy_bidirectional(&mut upgraded_client, &mut upgraded_upstream).await {
                                    error!("WS tunnel IO error for {domain} ({uri}): {e}", domain = domain_owned, uri = uri_owned, e = e);
                                }
                            }
                            Err(e) => {
                                error!("WS upstream upgrade failed for {domain} ({uri}): {e}", domain = domain_owned, uri = uri_owned, e = e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("WS client upgrade failed for {domain} ({uri}): {e}", domain = domain_owned, uri = uri_owned, e = e);
                    }
                }
            });

            Ok(response_to_client)
        }
        Err(e) => {
            let elapsed = start.elapsed();
            // Attempt DNS resolution for diagnostics
            let mut addrs: Vec<String> = Vec::new();
            if let Ok(iter) = tokio::net::lookup_host((upstream_host, upstream_port)).await {
                for a in iter {
                    addrs.push(a.ip().to_string());
                }
            }
            error!(
                "WS upstream request error for {domain} -> {uri} after {ms} ms: {e}; resolved_addrs={addrs:?}; note=TLS/SNI host='{host}' scheme='{scheme}'",
                domain = domain,
                uri = upstream_uri,
                ms = elapsed.as_millis(),
                e = e,
                addrs = addrs,
                host = upstream_host,
                scheme = upstream_scheme
            );
            Ok(Response::builder().status(StatusCode::BAD_GATEWAY).header("Content-Type", "text/plain").body(Body::from("Bad Gateway"))?)
        }
    }
}
