use std::time::Duration;
use std::sync::Arc;
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, ClientBuilder as ReqwestBuilder};
use http::{Method, HeaderMap, HeaderValue};
use url::Url;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::request::{Request, RequestBuilder};
use crate::response::Response;
use crate::cookies::CookieJar;
use crate::timeout::TimeoutConfig;
use crate::proxy::ProxyConfig;
use crate::tls::TlsConfig;
use crate::auth::AuthConfig;

/// Main HTTP client for RustTPX
///
/// This is the primary interface for making HTTP requests. It provides
/// both sync and async APIs, connection pooling, cookie persistence,
/// and comprehensive configuration options.
///
/// # Examples
///
/// ```rust
/// use rusttpx::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new();
///     let response = client.get("https://httpbin.org/json").send().await?;
///     println!("Status: {}", response.status());
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    inner: Arc<ReqwestClient>,
    cookie_jar: Arc<CookieJar>,
    timeout_config: TimeoutConfig,
    default_headers: HeaderMap,
    base_url: Option<Url>,
}

impl Client {
    /// Create a new client with default settings
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Create a new client builder
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Create a GET request
    pub fn get<U>(&self, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        self.request(Method::GET, url)
    }

    /// Create a POST request
    pub fn post<U>(&self, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        self.request(Method::POST, url)
    }

    /// Create a PUT request
    pub fn put<U>(&self, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        self.request(Method::PUT, url)
    }

    /// Create a DELETE request
    pub fn delete<U>(&self, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        self.request(Method::DELETE, url)
    }

    /// Create a PATCH request
    pub fn patch<U>(&self, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        self.request(Method::PATCH, url)
    }

    /// Create a HEAD request
    pub fn head<U>(&self, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        self.request(Method::HEAD, url)
    }

    /// Create a request with a custom method
    pub fn request<U>(&self, method: Method, url: U) -> RequestBuilder
    where
        U: Into<Url>,
    {
        let mut url = url.into();
        
        // Apply base URL if set
        if let Some(ref base_url) = self.base_url {
            url = base_url.join(url.as_str()).unwrap_or(url);
        }

        RequestBuilder::new(
            self.inner.clone(),
            self.cookie_jar.clone(),
            method,
            url,
            self.timeout_config.clone(),
            self.default_headers.clone(),
        )
    }

    /// Send a request and return the response
    pub async fn send(&self, request: Request) -> Result<Response> {
        let reqwest_response = self.inner
            .execute(request.into_reqwest_request()?)
            .await
            .map_err(Error::Network)?;

        Response::from_reqwest_response(reqwest_response, self.cookie_jar.clone()).await
    }

    /// Get the underlying reqwest client
    pub fn inner(&self) -> &ReqwestClient {
        &self.inner
    }

    /// Get the cookie jar
    pub fn cookie_jar(&self) -> &CookieJar {
        &self.cookie_jar
    }

    /// Get the timeout configuration
    pub fn timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }

    /// Get the base URL if set
    pub fn base_url(&self) -> Option<&Url> {
        self.base_url.as_ref()
    }

    /// Check if the client is closed
    pub fn is_closed(&self) -> bool {
        // Reqwest doesn't expose this, so we assume it's always open
        false
    }

    /// Close the client and free resources
    pub async fn close(&self) {
        // Reqwest handles cleanup automatically
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating HTTP clients with custom configuration
///
/// # Examples
///
/// ```rust
/// use rusttpx::ClientBuilder;
/// use std::time::Duration;
///
/// let client = ClientBuilder::new()
///     .timeout(Duration::from_secs(30))
///     .user_agent("MyApp/1.0")
///     .build();
/// ```
pub struct ClientBuilder {
    reqwest_builder: ReqwestBuilder,
    cookie_jar: Option<CookieJar>,
    timeout_config: TimeoutConfig,
    default_headers: HeaderMap,
    base_url: Option<Url>,
    proxy_config: Option<ProxyConfig>,
    tls_config: Option<TlsConfig>,
    auth_config: Option<AuthConfig>,
}

impl ClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self {
            reqwest_builder: ReqwestClient::builder(),
            cookie_jar: None,
            timeout_config: TimeoutConfig::default(),
            default_headers: HeaderMap::new(),
            base_url: None,
            proxy_config: None,
            tls_config: None,
            auth_config: None,
        }
    }

    /// Set the default timeout for all requests
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = TimeoutConfig::new(timeout);
        self.reqwest_builder = self.reqwest_builder.timeout(timeout);
        self
    }

    /// Set the connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.connect_timeout(timeout);
        self.reqwest_builder = self.reqwest_builder.connect_timeout(timeout);
        self
    }

    /// Set the read timeout
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.read_timeout(timeout);
        self
    }

    /// Set the write timeout
    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.write_timeout(timeout);
        self
    }

    /// Set the pool idle timeout
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.reqwest_builder = self.reqwest_builder.pool_idle_timeout(timeout);
        self
    }

    /// Set the maximum number of connections in the pool
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.reqwest_builder = self.reqwest_builder.pool_max_idle_per_host(max);
        self
    }

    /// Set a default header for all requests
    pub fn default_header(mut self, name: &str, value: &str) -> Result<Self> {
        let name = name.parse::<http::header::HeaderName>()?;
        let value = value.parse::<HeaderValue>()?;
        self.default_headers.insert(name, value);
        Ok(self)
    }

    /// Set the user agent
    pub fn user_agent(mut self, user_agent: &str) -> Result<Self> {
        self.default_header("User-Agent", user_agent)
    }

    /// Set the base URL for all requests
    pub fn base_url(mut self, url: impl Into<Url>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Enable or disable HTTP/2
    pub fn http2_prior_knowledge(mut self) -> Self {
        self.reqwest_builder = self.reqwest_builder.http2_prior_knowledge();
        self
    }

    /// Set the cookie jar
    pub fn cookie_jar(mut self, cookie_jar: CookieJar) -> Self {
        self.cookie_jar = Some(cookie_jar);
        self
    }

    /// Set proxy configuration
    pub fn proxy_config(mut self, config: ProxyConfig) -> Self {
        self.proxy_config = Some(config);
        self
    }

    /// Set TLS configuration
    pub fn tls_config(mut self, config: TlsConfig) -> Self {
        self.tls_config = Some(config);
        self
    }

    /// Set authentication configuration
    pub fn auth_config(mut self, config: AuthConfig) -> Self {
        self.auth_config = Some(config);
        self
    }

    /// Enable or disable automatic decompression
    // Note: reqwest doesn't have no_decompress method in this version
    // pub fn no_decompress(mut self) -> Self {
    //     self.reqwest_builder = self.reqwest_builder.no_decompress();
    //     self
    // }

    /// Set the maximum redirects to follow
    pub fn redirect(mut self, max_redirects: usize) -> Self {
        self.reqwest_builder = self.reqwest_builder.redirect(
            reqwest::redirect::Policy::limited(max_redirects)
        );
        self
    }

    /// Disable redirects
    pub fn no_redirect(mut self) -> Self {
        self.reqwest_builder = self.reqwest_builder.redirect(
            reqwest::redirect::Policy::none()
        );
        self
    }

    /// Set the referer policy
    pub fn referer(mut self, referer: bool) -> Self {
        self.reqwest_builder = self.reqwest_builder.referer(referer);
        self
    }

    /// Build the client
    pub fn build(self) -> Client {
        // Apply proxy configuration
        let reqwest_builder = if let Some(proxy_config) = self.proxy_config {
            proxy_config.apply_to_builder(self.reqwest_builder)
        } else {
            self.reqwest_builder
        };

        // Apply TLS configuration
        let reqwest_builder = if let Some(tls_config) = self.tls_config {
            tls_config.apply_to_builder(reqwest_builder)
        } else {
            reqwest_builder
        };

        // Build the reqwest client
        let reqwest_client = reqwest_builder
            .build()
            .expect("Failed to build reqwest client");

        // Create cookie jar
        let cookie_jar = self.cookie_jar.unwrap_or_else(CookieJar::new);

        Client {
            inner: Arc::new(reqwest_client),
            cookie_jar: Arc::new(cookie_jar),
            timeout_config: self.timeout_config,
            default_headers: self.default_headers,
            base_url: self.base_url,
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods for common HTTP operations
impl Client {
    /// Send a GET request and return JSON
    pub async fn get_json<T>(&self, url: impl Into<Url>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.get(url).send_json().await
    }

    /// Send a POST request with JSON body and return JSON
    pub async fn post_json<T, U>(&self, url: impl Into<Url>, body: &T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        self.post(url).json(body)?.send_json().await
    }

    /// Send a PUT request with JSON body and return JSON
    pub async fn put_json<T, U>(&self, url: impl Into<Url>, body: &T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        self.put(url).json(body)?.send_json().await
    }

    /// Send a PATCH request with JSON body and return JSON
    pub async fn patch_json<T, U>(&self, url: impl Into<Url>, body: &T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        self.patch(url).json(body)?.send_json().await
    }

    /// Send a DELETE request and return JSON
    pub async fn delete_json<T>(&self, url: impl Into<Url>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.delete(url).send_json().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = Client::new();
        assert!(!client.is_closed());
    }

    #[tokio::test]
    async fn test_client_builder() {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .user_agent("Test/1.0")
            .unwrap()
            .build();
        
        assert!(!client.is_closed());
    }

    #[tokio::test]
    async fn test_request_builder() {
        let client = Client::new();
        let request = client.get("https://httpbin.org/get");
        assert_eq!(request.method(), &Method::GET);
    }
} 