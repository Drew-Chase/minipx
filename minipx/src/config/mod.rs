// Configuration module
//
// This module contains all configuration-related functionality split into focused submodules:
// - types: Core configuration structures and types
// - loader: Configuration file loading and saving
// - validator: Configuration validation logic
// - manager: Global state management and broadcasting
// - watcher: File watching functionality

pub mod loader;
pub mod manager;
pub mod types;
pub mod validator;
pub mod watcher;

// Re-export main types for backward compatibility
pub use types::{Config, ProxyRoute, RoutePatch};
