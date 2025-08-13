use crate::config::Config;
use anyhow::{anyhow, Result};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, StatusCode};
use std::net::IpAddr;
use std::{convert::Infallible, net::SocketAddr};
use log::{error, info};

pub async fn start_rp_server() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 80));
    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle_request(remote_addr, req))) }
    });
    let server = hyper::Server::bind(&addr).serve(make_svc);
    info!("Reverse Proxy Server running on {}", addr);
    if let Err(e) = server.await {
        error!("Server error: {}", e);
    }
    Ok(())
}

async fn handle_request(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>> {
    let uri = req.uri();
    let domain = uri.host().ok_or(anyhow!("No host in URI"))?;
    let config = Config::get().await;
    let route = config.lookup_host(domain);
    if route.is_none() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain")
            .body(Body::from("Not Found"))?);
    }
    let port = route.unwrap();
    match hyper_reverse_proxy::call(
        client_ip,
        format!("http://127.0.0.1:{}", port).as_str(),
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
