use crate::config::Config;
use log::{error, info};
use std::collections::BTreeMap;
use std::net::SocketAddr;

/// Set up TCP/UDP forwarders for routes with custom listen ports
pub async fn setup_forwarders() {
    let config = Config::get().await;
    let mut listeners: BTreeMap<u16, (String, u16)> = BTreeMap::new();
    
    // Collect unique listen ports (excluding 80/443)
    for route in config.get_routes().values() {
        #[allow(clippy::collapsible_if)]
        if let Some(lp) = route.get_listen_port() {
            if lp != 0 && lp != 80 && lp != 443 {
                listeners.entry(lp).or_insert((route.get_host().to_string(), route.get_port()));
            }
        }
    }

    // Start forwarders for each unique port
    for (listen_port, (target_host, target_port)) in listeners {
        start_tcp_forwarder(listen_port, target_host.clone(), target_port);
        start_udp_forwarder(listen_port, target_host, target_port);
    }
}

/// Start a TCP forwarder that forwards connections from listen_port to target_host: target_port
fn start_tcp_forwarder(listen_port: u16, target_host: String, target_port: u16) {
    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
        loop {
            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    info!("TCP forwarder listening on {} -> {}:{}", addr, target_host, target_port);
                    loop {
                        match listener.accept().await {
                            Ok((mut inbound, peer)) => {
                                let host = target_host.clone();
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
}

/// Start a UDP forwarder that forwards packets from listen_port to target_host: target_port
fn start_udp_forwarder(listen_port: u16, target_host: String, target_port: u16) {
    tokio::spawn(async move {
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], listen_port));
        loop {
            match tokio::net::UdpSocket::bind(bind_addr).await {
                Ok(socket) => {
                    info!("UDP forwarder listening on {} -> {}:{}", bind_addr, target_host, target_port);
                    let upstream = (target_host.as_str(), target_port);
                    let mut buf = vec![0u8; 65535];
                    loop {
                        match socket.recv_from(&mut buf).await {
                            Ok((n, src)) => {
                                // send it to upstream
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