use crate::config::Config;
use anyhow::{anyhow, Result};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Request, Response, StatusCode};
use log::{error, info, warn};
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};
use tokio::sync::oneshot;

pub async fn start_rp_server() -> Result<()> {
    loop {
        let config = Config::get().await;
        let current_port = config.get_port();
        let addr = SocketAddr::from(([127, 0, 0, 1], current_port));

        let mut updates = Config::subscribe();

        let make_svc = make_service_fn(move |conn: &AddrStream| {
            let remote_addr = conn.remote_addr().ip();
            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let client_ip = remote_addr;
                    async move {
                        match handle_request(client_ip, req).await {
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
                loop {
                    match updates.recv().await {
                        Ok(updated) if updated.get_port() != current_port => break,
                        Ok(_) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            warn!("Config update channel closed; stopping server supervisor");
                            return Ok(());
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Missed {n} config updates while waiting to rebind");
                        }
                    }
                }
                continue;
            }
        };

        let server = builder.serve(make_svc);

        info!("Reverse Proxy Server running on {}", addr);

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let graceful = server.with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });
        tokio::pin!(graceful);
        loop {
            tokio::select! {
                res = &mut graceful => {
                    if let Err(e) = res {
                        error!("Server error: {}", e);
                    } else {
                        info!("Server on {} stopped", addr);
                    }
                    break;
                }
                Ok(updated) = updates.recv() => {
                    if updated.get_port() != current_port {
                        info!("Port changed {} -> {}; restarting server", current_port, updated.get_port());
                        let _ = shutdown_tx.send(());
                        if let Err(e) = (&mut graceful).await {
                            error!("Error while shutting down server on {}: {}", addr, e);
                        }
                        break;
                    }
                }
            }
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

async fn handle_request(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>> {
    let uri = req.uri().clone();
    let domain = extract_host(&req).ok_or(anyhow!("No host in URI or Host header"))?;

    let config = Config::get().await;
    let route = config.lookup_host(&domain);

    if route.is_none() {
        warn!("Received request from {ip} for unknown host {host}", ip=client_ip, host=domain);
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(Body::from("Not Found"))?);
    }

    let route = route.unwrap();
    info!("Received request from {ip} for {host} redirecting to 127.0.0.1:{port:?}{path}", ip=client_ip, path=uri, host=domain, port=route);
    match hyper_reverse_proxy::call(
        client_ip,
        route.as_str(),
        req,
    )
    .await
    {
        Ok(response) => Ok(response),
        Err(_error) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())?),
    }
}
