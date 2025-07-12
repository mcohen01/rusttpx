use std::sync::Arc;
use futures::Stream;
use reqwest::{Response as ReqwestResponse, StatusCode};
use http::{HeaderMap, HeaderValue};
use serde_json::Value;

use crate::error::{Error, Result, StatusError};
use crate::cookies::CookieJar;

/// HTTP response representation
///
/// This type represents an HTTP response received from a server.
/// It provides methods for accessing response properties and reading the body.
#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    url: url::Url,
    version: http::Version,
    inner: ReqwestResponse,
    cookie_jar: Arc<CookieJar>,
}

impl Response {
    /// Create a response from a reqwest response
    pub async fn from_reqwest_response(
        reqwest_response: ReqwestResponse,
        cookie_jar: Arc<CookieJar>,
    ) -> Result<Self> {
        // Extract cookies from response headers
        if let Some(cookie_header) = reqwest_response.headers().get("set-cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                cookie_jar.add_cookie_from_response(cookie_str, &reqwest_response.url());
            }
        }

        let status = reqwest_response.status();
        let headers = reqwest_response.headers().clone();
        let url = reqwest_response.url().clone();
        let version = reqwest_response.version();

        Ok(Self {
            status,
            headers,
            url,
            version,
            inner: reqwest_response,
            cookie_jar,
        })
    }

    /// Get the HTTP status code
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the HTTP version
    pub fn version(&self) -> http::Version {
        self.version
    }

    /// Get the response headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a specific header value
    pub fn header(&self, name: &str) -> Option<&HeaderValue> {
        self.headers.get(name)
    }

    /// Get the content type
    pub fn content_type(&self) -> Option<&str> {
        self.headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
    }

    /// Get the content length
    pub fn content_length(&self) -> Option<u64> {
        self.headers
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
    }

    /// Get the URL that was requested
    pub fn url(&self) -> &url::Url {
        &self.url
    }

    /// Check if the response is successful (2xx status code)
    pub fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the response is a client error (4xx status code)
    pub fn is_client_error(&self) -> bool {
        self.status.is_client_error()
    }

    /// Check if the response is a server error (5xx status code)
    pub fn is_server_error(&self) -> bool {
        self.status.is_server_error()
    }

    /// Check if the response indicates a redirect
    pub fn is_redirect(&self) -> bool {
        self.status.is_redirection()
    }

    /// Check if the response indicates an informational status
    pub fn is_informational(&self) -> bool {
        self.status.is_informational()
    }

    /// Raise an error for bad status codes
    pub fn error_for_status(self) -> Result<Self> {
        if self.status.is_client_error() {
            return Err(Error::from(StatusError::client(
                self.status,
                format!("Client error: {}", self.status),
            )));
        }
        if self.status.is_server_error() {
            return Err(Error::from(StatusError::server(
                self.status,
                format!("Server error: {}", self.status),
            )));
        }
        Ok(self)
    }

    /// Raise an error for bad status codes (consumes self)
    pub fn error_for_status_ref(&self) -> Result<&Self> {
        if self.status.is_client_error() {
            return Err(Error::from(StatusError::client(
                self.status,
                format!("Client error: {}", self.status),
            )));
        }
        if self.status.is_server_error() {
            return Err(Error::from(StatusError::server(
                self.status,
                format!("Server error: {}", self.status),
            )));
        }
        Ok(self)
    }

    /// Get the response body as text
    pub async fn text(self) -> Result<String> {
        self.inner
            .text()
            .await
            .map_err(Error::Network)
    }

    /// Get the response body as bytes
    pub async fn bytes(self) -> Result<Vec<u8>> {
        self.inner
            .bytes()
            .await
            .map_err(Error::Network)
            .map(|b| b.to_vec())
    }

    /// Get the response body as JSON
    pub async fn json<T>(self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.inner
            .json()
            .await
            .map_err(Error::Network)
    }

    /// Get the response body as a stream of bytes
    pub fn bytes_stream(self) -> impl Stream<Item = Result<Vec<u8>>> {
        use futures::StreamExt;
        self.inner
            .bytes_stream()
            .map(|chunk| chunk.map(|b| b.to_vec()).map_err(|e| Error::Network(e)))
    }

    /// Get the response body as a stream of text chunks
    // Note: reqwest::Response doesn't have text_stream method in this version
    // pub fn text_stream(self) -> impl Stream<Item = Result<String>> {
    //     self.inner
    //         .text_stream()
    //         .map(|chunk| chunk.map_err(Error::Network))
    // }

    // Note: reqwest::Response doesn't have json_stream method in this version
    // pub fn json_stream<T>(self) -> impl Stream<Item = Result<T>>
    // where
    //     T: serde::de::DeserializeOwned,
    // {
    //     self.inner
    //         .json_stream()
    //         .map(|chunk| chunk.map_err(Error::Network))
    // }

    /// Copy the response body to a writer
    pub async fn copy_to<W>(self, writer: &mut W) -> Result<u64>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        let bytes = self.bytes().await?;
        use tokio::io::AsyncWriteExt;
        writer.write_all(&bytes).await.map_err(|e| Error::Custom(format!("IO error: {}", e)))?;
        Ok(bytes.len() as u64)
    }

    // Note: reqwest::Response doesn't implement AsyncRead in this version
    // pub fn reader(self) -> impl AsyncRead {
    //     self.inner
    // }

    /// Get the underlying reqwest response
    pub fn into_inner(self) -> ReqwestResponse {
        self.inner
    }

    /// Get a reference to the underlying reqwest response
    pub fn inner(&self) -> &ReqwestResponse {
        &self.inner
    }

    /// Get a mutable reference to the underlying reqwest response
    pub fn inner_mut(&mut self) -> &mut ReqwestResponse {
        &mut self.inner
    }

    /// Get the cookie jar
    pub fn cookie_jar(&self) -> &CookieJar {
        &self.cookie_jar
    }

    /// Get the effective URL (after redirects)
    pub fn effective_url(&self) -> Option<&url::Url> {
        Some(self.inner.url())
    }

    /// Get the remote address
    pub fn remote_addr(&self) -> Option<std::net::SocketAddr> {
        self.inner.remote_addr()
    }

    /// Get the response extensions
    pub fn extensions(&self) -> &http::Extensions {
        self.inner.extensions()
    }

    /// Get mutable access to response extensions
    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        self.inner.extensions_mut()
    }
}

impl Clone for Response {
    fn clone(&self) -> Self {
        // Note: reqwest::Response doesn't support cloning in this version
        // We'll create a new response with the same metadata
        panic!("Response cloning is not supported in this version of reqwest")
    }
}

/// Response builder for creating mock responses
pub struct ResponseBuilder {
    status: StatusCode,
    headers: HeaderMap,
    url: url::Url,
    version: http::Version,
    body: Option<Vec<u8>>,
}

impl ResponseBuilder {
    /// Create a new response builder
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            url: "http://localhost/".parse().unwrap(),
            version: http::Version::HTTP_11,
            body: None,
        }
    }

    /// Set the status code
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    /// Set the headers
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Set a header
    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        let name = name.parse::<http::header::HeaderName>()?;
        let value = value.parse::<HeaderValue>()?;
        self.headers.insert(name, value);
        Ok(self)
    }

    /// Set the content type
    pub fn content_type(self, content_type: &str) -> Result<Self> {
        self.header("Content-Type", content_type)
    }

    /// Set the URL
    pub fn url(mut self, url: url::Url) -> Self {
        self.url = url;
        self
    }

    /// Set the HTTP version
    pub fn version(mut self, version: http::Version) -> Self {
        self.version = version;
        self
    }

    /// Set the body
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set text body
    pub fn text(mut self, text: &str) -> Self {
        self.body = Some(text.as_bytes().to_vec());
        self
    }

    /// Set JSON body
    pub fn json(mut self, json: &Value) -> Result<Self> {
        let json_bytes = serde_json::to_vec(json)?;
        self.body = Some(json_bytes);
        Ok(self)
    }

    /// Build the response
    pub fn build(self) -> Result<Response> {
        // Note: reqwest::Response::new is private in this version
        // We'll create a simple response without the inner reqwest response
        // This is a limitation of the current reqwest version
        
        // Create cookie jar
        let _cookie_jar = Arc::new(CookieJar::new());

        // For now, we'll return an error since we can't create a proper reqwest response
        Err(Error::custom("ResponseBuilder::build is not supported in this version of reqwest"))
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new(StatusCode::OK)
    }
}

/// Convenience methods for common response operations
impl Response {
    /// Create a response builder
    pub fn builder(status: StatusCode) -> ResponseBuilder {
        ResponseBuilder::new(status)
    }

    /// Create a successful response builder
    pub fn ok() -> ResponseBuilder {
        ResponseBuilder::new(StatusCode::OK)
    }

    /// Create a not found response builder
    pub fn not_found() -> ResponseBuilder {
        ResponseBuilder::new(StatusCode::NOT_FOUND)
    }

    /// Create an internal server error response builder
    pub fn internal_server_error() -> ResponseBuilder {
        ResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_response_creation() {
        let builder = ResponseBuilder::new(StatusCode::OK)
            .content_type("application/json")
            .unwrap()
            .text(r#"{"message": "Hello, World!"}"#);
        
        let response = builder.build().unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.content_type(), Some("application/json"));
    }

    #[test]
    fn test_response_status_checks() {
        let response = ResponseBuilder::new(StatusCode::OK).build().unwrap();
        assert!(response.is_success());
        assert!(!response.is_client_error());
        assert!(!response.is_server_error());
        assert!(!response.is_redirect());

        let response = ResponseBuilder::new(StatusCode::NOT_FOUND).build().unwrap();
        assert!(!response.is_success());
        assert!(response.is_client_error());
        assert!(!response.is_server_error());

        let response = ResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).build().unwrap();
        assert!(!response.is_success());
        assert!(!response.is_client_error());
        assert!(response.is_server_error());
    }
} 