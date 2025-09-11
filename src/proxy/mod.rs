// Proxy module
//
// This module contains all reverse proxy functionality split into focused submodules:
// - http_server: HTTP server setup and management
// - https_server: HTTPS/SSL server functionality (from ssl_server.rs)
// - request_handler: HTTP request processing logic
// - websocket: WebSocket handling logic
// - forwarder: TCP/UDP forwarding logic

pub mod http_server;
pub mod request_handler;
pub mod websocket;
pub mod forwarder;

// Re-export main function for backward compatibility
pub use http_server::start_rp_server;