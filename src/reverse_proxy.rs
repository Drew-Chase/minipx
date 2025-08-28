use crate::config::Config;
use anyhow::{Result, anyhow};
use hyper::Client;
use hyper::body::to_bytes;
use hyper::http::Version;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade;
use hyper::{Body, Request, Response, StatusCode, header};
use hyper_tls::HttpsConnector;
use log::{debug, error, info, warn};
use std::net::IpAddr;
use std::time::Instant;
use std::{convert::Infallible, net::SocketAddr};

pub async fn start_rp_server() -> Result<()> {
    // Spawn TCP/UDP forwarders for any routes that specify a custom listen_port (excluding 80/443)
    {
        let config = Config::get().await;
        use std::collections::BTreeMap;
        let mut listeners: BTreeMap<u16, (String, u16)> = BTreeMap::new();
        for route in config.get_routes().values() {
            #[allow(clippy::collapsible_if)]
            if let Some(lp) = route.get_listen_port() {
                if lp != 0 && lp != 80 && lp != 443 {
                    listeners.entry(lp).or_insert((route.get_host().to_string(), route.get_port()));
                }
            }
        }
        for (listen_port, (target_host, target_port)) in listeners {
            // TCP forwarder
            let host_tcp = target_host.clone();
            tokio::spawn(async move {
                let addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
                loop {
                    match tokio::net::TcpListener::bind(addr).await {
                        Ok(listener) => {
                            info!("TCP forwarder listening on {} -> {}:{}", addr, host_tcp, target_port);
                            loop {
                                match listener.accept().await {
                                    Ok((mut inbound, peer)) => {
                                        let host = host_tcp.clone();
                                        tokio::spawn(async move {
                                            match tokio::net::TcpStream::connect((host.as_str(), target_port)).await {
                                                Ok(mut outbound) => {
                                                    let _ = tokio::io::copy_bidirectional(&mut inbound, &mut outbound).await;
                                                }
                                                Err(e) => {
                                                    error!("TCP forward connect failed from {} to {}:{}: {}", peer, host, target_port, e);
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        error!("TCP accept error on {}: {}", addr, e);
                                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to bind TCP forwarder on {}: {}", addr, e);
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                }
            });

            // UDP forwarder (best-effort)
            let host_udp = target_host.clone();
            tokio::spawn(async move {
                let bind_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
                loop {
                    match tokio::net::UdpSocket::bind(bind_addr).await {
                        Ok(socket) => {
                            info!("UDP forwarder listening on {} -> {}:{}", bind_addr, host_udp, target_port);
                            let upstream = (host_udp.as_str(), target_port);
                            let mut buf = vec![0u8; 65535];
                            loop {
                                match socket.recv_from(&mut buf).await {
                                    Ok((n, src)) => {
                                        // send to upstream
                                        if let Err(e) = socket.send_to(&buf[..n], upstream).await {
                                            error!("UDP send_to upstream failed: {}", e);
                                            continue;
                                        }
                                        // try to read a response and send back
                                        let mut resp_buf = vec![0u8; 65535];
                                        if let Ok(Ok((rn, _up))) =
                                            tokio::time::timeout(std::time::Duration::from_millis(200), socket.recv_from(&mut resp_buf)).await
                                        {
                                            let _ = socket.send_to(&resp_buf[..rn], src).await;
                                        }
                                    }
                                    Err(e) => {
                                        error!("UDP recv_from error on {}: {}", bind_addr, e);
                                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to bind UDP forwarder on {}: {}", bind_addr, e);
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            continue;
                        }
                    }
                }
            });
        }
    }

    // Default HTTP listener on port 80
    loop {
        let addr = SocketAddr::from(([0, 0, 0, 0], 80));

        let make_svc = make_service_fn(move |conn: &AddrStream| {
            let remote_addr = conn.remote_addr().ip();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let client_ip = remote_addr;
                    async move {
                        match handle_request_with_scheme("http", client_ip, req).await {
                            Ok(resp) => Ok::<_, Infallible>(resp),
                            Err(e) => {
                                error!("handle_request error from {}: {}", client_ip, e);
                                Ok::<_, Infallible>(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap())
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

    #[allow(clippy::collapsible_if)]
    if let Some(hv) = req.headers().get(header::HOST) {
        if let Ok(host) = hv.to_str() {
            let host_only = host.split(':').next().unwrap_or(host);
            return Some(host_only.to_string());
        }
    }
    req.uri().host().map(|h| h.to_string())
}

fn is_websocket(req: &Request<Body>) -> bool {
    let has_upgrade_ws =
        req.headers().get(header::UPGRADE).and_then(|v| v.to_str().ok()).map(|v| v.eq_ignore_ascii_case("websocket")).unwrap_or(false);
    let has_connection_upgrade =
        req.headers().get(header::CONNECTION).and_then(|v| v.to_str().ok()).map(|v| v.to_ascii_lowercase().contains("upgrade")).unwrap_or(false);
    has_upgrade_ws && has_connection_upgrade
}

pub async fn handle_request_with_scheme(frontend_scheme: &str, client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>> {
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

    let path = route.get_path();
    let target = format!("{}://{}:{}{}", upstream_scheme, route.get_host(), route.get_port(), path);
    info!(
        "Received request from {ip} for {fs}://{host} -> {route}{req_path}",
        fs = frontend_scheme,
        ip = client_ip,
        req_path = uri,
        host = domain,
        route = target
    );
    if is_websocket(&req) {
        debug!("WS upstream scheme selected: {scheme}", scheme = upstream_scheme);
    }

    if is_websocket(&req) {
        debug!("WebSocket upgrade detected: frontend={fs}, upstream={up}", fs = frontend_scheme, up = target);
        return proxy_websocket(client_ip, req, upstream_scheme, route.get_host(), route.get_port(), route.get_path(), &domain).await;
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

async fn proxy_websocket(
    client_ip: IpAddr,
    req: Request<Body>,
    upstream_scheme: &str,
    upstream_host: &str,
    upstream_port: u16,
    upstream_base_path: &str,
    domain: &str,
) -> Result<Response<Body>> {
    // Build upstream URI: base path + requested path_and_query
    let suffix = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/");

    // For WebSocket upgrades, always use http:// for upstream connections
    // TLS is terminated at the proxy, so backend connections are plain HTTP
    let upstream_uri = format!("http://{}:{}{}{}", upstream_host, upstream_port, upstream_base_path, suffix);

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
