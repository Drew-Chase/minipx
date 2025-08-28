use crate::config::Config;
use crate::reverse_proxy::handle_request_with_scheme;
use anyhow::Result;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use log::{error, info, warn};
use rustls_acme::AcmeConfig;
use rustls_acme::caches::DirCache;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::TcpListenerStream;

pub async fn start_ssl_server() -> Result<()> {
    loop {
        let config = Config::get().await;

        // Respect global SSL enable flag
        if !config.is_ssl_enabled() {
            warn!("SSL is disabled via config; HTTPS server will wait for enablement");
            let mut updates = Config::subscribe();
            loop {
                // Wait for a message from the config channel
                // and check if SSL is enabled
                match updates.recv().await {
                    Ok(updated) if updated.is_ssl_enabled() => break,
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        warn!("Config update channel closed; stopping HTTPS server supervisor");
                        return Ok(());
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Missed {n} config updates while waiting for SSL enable")
                    }
                }
            }
            continue; // restart the main loop.
        }

        // Validate email (global)
        if !config.is_email_valid() {
            warn!("Invalid ACME email in config; HTTPS server will wait for a valid email");
            let mut updates = Config::subscribe();
            loop {
                match updates.recv().await {
                    Ok(updated) if updated.is_email_valid() && updated.is_ssl_enabled() => break,
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        warn!("Config update channel closed; stopping HTTPS server supervisor");
                        return Ok(());
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Missed {n} config updates while waiting for valid email")
                    }
                }
            }
            continue;
        }

        // Validate domains (per-route); run with valid subset, skip invalid
        let (valid_domains, invalid_domains) = config.get_valid_domains_for_acme();
        if !invalid_domains.is_empty() {
            warn!("Invalid ACME domains will be skipped: {:?}", invalid_domains);
        }
        if valid_domains.is_empty() {
            warn!("No valid domains configured for ACME; HTTPS server will wait for config updates");
            let mut updates = Config::subscribe();
            loop {
                match updates.recv().await {
                    Ok(updated) => {
                        if updated.is_ssl_enabled() && updated.is_email_valid() {
                            let (vd, _) = updated.get_valid_domains_for_acme();
                            if !vd.is_empty() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        warn!("Config update channel closed; stopping HTTPS server supervisor");
                        return Ok(());
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Missed {n} config updates while waiting for valid domains")
                    }
                }
            }
            continue;
        }

        let email = config.get_email().clone();
        let cache_dir = config.get_cache_dir().clone();
        if let Err(e) = tokio::fs::create_dir_all(&cache_dir).await {
            warn!("Failed to create cache_dir {}: {}", cache_dir, e);
        }

        // Bind to [::]:443 (all interfaces)
        let addr = (std::net::Ipv6Addr::UNSPECIFIED, 443);
        let tcp_listener = match TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind HTTPS server on [::]:443: {}", e);
                let mut updates = Config::subscribe();
                loop {
                    match updates.recv().await {
                        Ok(_) => break, // on any update try again (port fixed)
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            warn!("Config update channel closed; stopping HTTPS server supervisor");
                            return Ok(());
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Missed {n} config updates while waiting to rebind HTTPS")
                        }
                    }
                }
                continue;
            }
        };
        let tcp_incoming = TcpListenerStream::new(tcp_listener);

        // Configure ACME with Let's Encrypt production directory and DirCache, build TLS incoming stream
        let tls_incoming = AcmeConfig::new(valid_domains.clone())
            .contact_push(format!("mailto:{}", email))
            .cache(DirCache::new(cache_dir.clone()))
            .directory_lets_encrypt(true)
            .tokio_incoming(tcp_incoming, Vec::new());

        info!("HTTPS Server (ACME) running on [::]:443 for domains: {:?}", valid_domains);

        // Setup graceful shutdown
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        // Service factory for HTTPS requests
        let make_svc = |remote_ip: std::net::IpAddr, req: Request<Body>| async move {
            match handle_request_with_scheme("https", remote_ip, req).await {
                Ok(resp) => Ok::<Response<Body>, std::convert::Infallible>(resp),
                Err(e) => {
                    error!("HTTPS handle_request error from {}: {}", remote_ip, e);
                    Ok::<Response<Body>, std::convert::Infallible>(Response::new(Body::empty()))
                }
            }
        };

        // Spawn accept loop (own the stream inside the task)
        let server_task = tokio::spawn(async move {
            let mut tls_incoming = tls_incoming;
            let mut shutdown_rx = shutdown_rx;
            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    incoming = tls_incoming.next() => {
                        match incoming {
                            Some(Ok(tls)) => {
                                // Peer address is not available via high-level API; fall back to loopback for logging/XFF
                                let client_ip = std::net::IpAddr::from([127,0,0,1]);
                                tokio::spawn(async move {
                                    let service = service_fn(move |req| make_svc(client_ip, req));
                                    let mut http = hyper::server::conn::Http::new();
                                    http.http1_only(true);
                                    http.http1_keep_alive(true);
                                    let conn = http.serve_connection(tls, service).with_upgrades();
                                    if let Err(e) = conn.await {
                                        error!("HTTPS connection error: {}", e);
                                    }
                                });
                            }
                            Some(Err(e)) => {
                                warn!("TLS incoming error: {}", e);
                                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                            }
                            None => {
                                warn!("TLS incoming stream ended");
                                break;
                            }
                        }
                    }
                }
            }
        });

        // Watch for config updates that require restart (domains, email, cache_dir)
        let mut updates = Config::subscribe();
        loop {
            match updates.recv().await {
                Ok(updated) => {
                    let (new_valid, _new_invalid) = updated.get_valid_domains_for_acme();
                    let should_restart = !updated.is_ssl_enabled()
                        || !updated.is_email_valid()
                        || new_valid != valid_domains
                        || *updated.get_email() != email
                        || *updated.get_cache_dir() != cache_dir;
                    if should_restart {
                        info!("SSL config changed; restarting HTTPS server to apply updates");
                        let _ = shutdown_tx.send(());
                        let _ = server_task.await;
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    warn!("Config update channel closed; stopping HTTPS server supervisor");
                    let _ = shutdown_tx.send(());
                    let _ = server_task.await;
                    return Ok(());
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Missed {n} config updates while running HTTPS server");
                }
            }
        }
    }
}
