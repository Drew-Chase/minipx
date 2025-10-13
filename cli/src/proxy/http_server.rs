use crate::proxy::request_handler::handle_request_with_scheme;
use crate::proxy::forwarder::setup_forwarders;
use anyhow::Result;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode};
use log::{error, info};
use std::convert::Infallible;
use std::net::SocketAddr;

/// Start the reverse proxy server with HTTP support on port 80
pub async fn start_rp_server() -> Result<()> {
    // Set up TCP/UDP forwarders for custom listen ports
    setup_forwarders().await;

    // Start an HTTP server on port 80
    start_http_server().await
}

/// Start the HTTP server on port 80
async fn start_http_server() -> Result<()> {
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
                                Ok::<_, Infallible>(Response::builder()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(Body::empty())
                                    .unwrap())
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