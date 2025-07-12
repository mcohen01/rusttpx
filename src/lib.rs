//! RustTPX - A next-generation HTTP client for Rust
//! 
//! Inspired by Python's HTTPX library, RustTPX provides a modern, async-first HTTP client
//! with support for HTTP/1.1, HTTP/2, and comprehensive features for making HTTP requests.
//!
//! ## Features
//!
//! - **Async-first design** with both sync and async APIs
//! - **HTTP/1.1 and HTTP/2 support**
//! - **Request/Response compatibility** with standard HTTP traits
//! - **Strict timeouts** everywhere
//! - **Connection pooling** and keep-alive
//! - **Cookie persistence** across requests
//! - **SSL/TLS verification** with native certificates
//! - **Automatic compression** handling
//! - **Multipart file uploads**
//! - **Proxy support**
//! - **Streaming downloads**
//! - **Type-safe JSON** handling with Serde
//!
//! ## Quick Start
//!
//! ```rust
//! use rusttpx::{Client, Response};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!     let response: Response = client.get("https://httpbin.org/json").send().await?;
//!     
//!     println!("Status: {}", response.status());
//!     println!("Body: {}", response.text().await?);
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod request;
pub mod response;
pub mod error;
pub mod transport;
pub mod middleware;
pub mod auth;
pub mod cookies;
pub mod multipart;
pub mod streaming;
pub mod timeout;
pub mod proxy;
pub mod tls;

// Re-export main types for convenience
pub use client::{Client, ClientBuilder};
pub use request::{Request, RequestBuilder};
pub use response::Response;
pub use error::{Error, Result};

// Re-export common HTTP types
pub use http::{Method, StatusCode, HeaderMap, HeaderValue, Uri};

// Re-export async runtime for convenience
pub use tokio;

// Re-export JSON types
pub use serde_json::{Value as JsonValue, Map as JsonMap};

// Re-export URL types
pub use url::Url;

// Re-export time types
pub use std::time::Duration;

// Re-export common traits
pub use async_trait::async_trait;

// Module for internal use only
mod internal {

} 