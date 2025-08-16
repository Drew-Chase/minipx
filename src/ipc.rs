use interprocess::local_socket::prelude::{LocalSocketListener, LocalSocketStream};
use interprocess::local_socket::traits::{ListenerExt, Stream as _};
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Name, ToNsName};
use log::{debug, trace, warn};
use std::path::PathBuf;

const SOCKET_NAME: &str = "minipx_config_path_v1"; // Cross-platform local socket / named pipe name

/// Try to retrieve the config path from a running minipx instance via local IPC.
/// Returns None if no instance is listening.
pub async fn get_running_config_path() -> Option<String> {
    // Prefer namespaced name for Windows/Linux abstract namespace; falls back as per crate.
    let name: Name = match SOCKET_NAME.to_ns_name::<GenericNamespaced>() {
        Ok(n) => n,
        Err(_) => return None,
    };
    tokio::task::spawn_blocking(move || match LocalSocketStream::connect(name) {
        Ok(mut stream) => {
            use std::io::Read;
            let mut buf = Vec::with_capacity(256);
            if let Err(e) = stream.read_to_end(&mut buf) {
                warn!("IPC read error: {}", e);
                return None;
            }
            let s = String::from_utf8_lossy(&buf).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        }
        Err(_e) => None,
    })
    .await
    .ok()
    .flatten()
}

/// Start a background IPC listener that serves the config path to any connector.
/// Best-effort; if binding fails (another instance active), we log and return.
pub fn start_ipc_server(config_path: PathBuf) {
    std::thread::spawn(move || {
        let name: Name = match SOCKET_NAME.to_ns_name::<GenericNamespaced>() {
            Ok(n) => n,
            Err(_) => return,
        };
        let listener: LocalSocketListener = match ListenerOptions::new().name(name.borrow()).create_sync() {
            Ok(l) => l,
            Err(e) => {
                warn!("IPC server bind failed (likely already running): {}", e);
                return;
            }
        };
        debug!("IPC server listening on '{}'", SOCKET_NAME);
        for conn in listener.incoming() {
            match conn {
                Ok(mut stream) => {
                    use std::io::Write;
                    trace!("IPC client connected, sending config path");
                    let payload = config_path.to_string_lossy().into_owned();
                    let _ = stream.write_all(payload.as_bytes());
                    let _ = stream.flush();
                }
                Err(e) => {
                    warn!("IPC accept failed: {}", e);
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    });
}
