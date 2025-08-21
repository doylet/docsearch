/// HTTP infrastructure module
/// 
/// This module contains the HTTP server implementation using Axum,
/// including route handlers, middleware, and server configuration.

pub mod handlers;
pub mod server;

// Re-export commonly used types
pub use server::{HttpServer, ServerConfig};
