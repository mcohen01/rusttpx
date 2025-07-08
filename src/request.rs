use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use reqwest::{Request as ReqwestRequest, RequestBuilder as ReqwestBuilder};
use http::{Method, HeaderMap, HeaderValue, Uri};
use url::Url;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::response::Response;
use crate::cookies::CookieJar;
use crate::timeout::TimeoutConfig;

/// HTTP request representation
///
/// This type represents an HTTP request that can be sent by the client.
/// It provides methods for accessing request properties and sending the request.
#[derive(Clone)]
pub struct Request {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<RequestBody>,
    timeout_config: TimeoutConfig,
}

/// Request body types
#[derive(Clone)]
pub enum RequestBody {
    /// Empty body
    Empty,
    /// String body
    Text(String),
    /// JSON body
    Json(Value),
    /// Bytes body
    Bytes(Vec<u8>),
    /// Form data
    Form(Vec<(String, String)>),
    /// Multipart form data
    Multipart(Vec<(String, MultipartPart)>),
}

/// Multipart form part
#[derive(Clone)]
pub struct MultipartPart {
    pub name: String,
    pub content: MultipartContent,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

/// Multipart content types
#[derive(Clone)]
pub enum MultipartContent {
    Text(String),
    File(Vec<u8>),
}

impl Request {
    /// Create a new request
    pub fn new(method: Method, url: Url) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: Some(RequestBody::Empty),
            timeout_config: TimeoutConfig::default(),
        }
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get mutable access to headers
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the body
    pub fn body(&self) -> Option<&RequestBody> {
        self.body.as_ref()
    }

    /// Get the timeout configuration
    pub fn timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }

    /// Set a header
    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        let name = name.parse::<http::header::HeaderName>()?;
        let value = value.parse::<HeaderValue>()?;
        self.headers.insert(name, value);
        Ok(self)
    }

    /// Set the content type
    pub fn content_type(mut self, content_type: &str) -> Result<Self> {
        self.header("Content-Type", content_type)
    }

    /// Set the user agent
    pub fn user_agent(mut self, user_agent: &str) -> Result<Self> {
        self.header("User-Agent", user_agent)
    }

    /// Set the authorization header
    pub fn authorization(mut self, auth: &str) -> Result<Self> {
        self.header("Authorization", auth)
    }

    /// Set the accept header
    pub fn accept(mut self, accept: &str) -> Result<Self> {
        self.header("Accept", accept)
    }

    /// Set JSON body
    pub fn json<T>(mut self, body: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        let json = serde_json::to_value(body)?;
        self.body = Some(RequestBody::Json(json));
        self = self.content_type("application/json")?;
        Ok(self)
    }

    /// Set text body
    pub fn text(mut self, body: &str) -> Result<Self> {
        self.body = Some(RequestBody::Text(body.to_string()));
        self = self.content_type("text/plain")?;
        Ok(self)
    }

    /// Set bytes body
    pub fn bytes(mut self, body: Vec<u8>) -> Result<Self> {
        self.body = Some(RequestBody::Bytes(body));
        Ok(self)
    }

    /// Set form data
    pub fn form(mut self, data: Vec<(String, String)>) -> Result<Self> {
        self.body = Some(RequestBody::Form(data));
        self = self.content_type("application/x-www-form-urlencoded")?;
        Ok(self)
    }

    /// Set multipart form data
    pub fn multipart(mut self, parts: Vec<(String, MultipartPart)>) -> Result<Self> {
        self.body = Some(RequestBody::Multipart(parts));
        // Don't set content type for multipart, it will be set automatically
        Ok(self)
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.timeout(timeout);
        self
    }

    /// Convert to reqwest request
    pub fn into_reqwest_request(self) -> Result<ReqwestRequest> {
        let mut builder = ReqwestRequest::new(self.method, self.url.into());
        
        // Set headers
        for (name, value) in self.headers {
            if let Some(name) = name {
                builder.headers_mut().insert(name, value);
            }
        }

        // Set body
        match self.body {
            Some(RequestBody::Empty) => {
                // No body needed
            }
            Some(RequestBody::Text(text)) => {
                *builder.body_mut() = Some(text.into());
            }
            Some(RequestBody::Json(json)) => {
                let json_bytes = serde_json::to_vec(&json)?;
                *builder.body_mut() = Some(json_bytes.into());
            }
            Some(RequestBody::Bytes(bytes)) => {
                *builder.body_mut() = Some(bytes.into());
            }
            Some(RequestBody::Form(data)) => {
                let form_data = url::form_urlencoded::Serializer::new(String::new())
                    .extend_pairs(data)
                    .finish();
                *builder.body_mut() = Some(form_data.into());
            }
            Some(RequestBody::Multipart(_)) => {
                // Multipart needs special handling in the builder
                return Err(Error::custom("Multipart requests must be built with RequestBuilder"));
            }
            None => {
                // No body
            }
        }

        Ok(builder)
    }
}

/// Builder for creating HTTP requests
///
/// This provides a fluent interface for building requests with various
/// configurations, headers, and body types.
pub struct RequestBuilder {
    reqwest_builder: ReqwestBuilder,
    cookie_jar: Arc<CookieJar>,
    method: Method,
    url: Url,
    timeout_config: TimeoutConfig,
    default_headers: HeaderMap,
}

impl RequestBuilder {
    /// Create a new request builder
    pub fn new(
        reqwest_client: Arc<reqwest::Client>,
        cookie_jar: Arc<CookieJar>,
        method: Method,
        url: Url,
        timeout_config: TimeoutConfig,
        default_headers: HeaderMap,
    ) -> Self {
        let reqwest_builder = reqwest_client.request(method.clone(), url.as_str());
        
        Self {
            reqwest_builder,
            cookie_jar,
            method,
            url,
            timeout_config,
            default_headers,
        }
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get the URL
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Set a header
    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        let name = name.parse::<http::header::HeaderName>()?;
        let value = value.parse::<HeaderValue>()?;
        self.reqwest_builder = self.reqwest_builder.header(name, value);
        Ok(self)
    }

    /// Set multiple headers
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        for (name, value) in headers {
            if let Some(name) = name {
                self.reqwest_builder = self.reqwest_builder.header(name, value);
            }
        }
        self
    }

    /// Set the content type
    pub fn content_type(mut self, content_type: &str) -> Result<Self> {
        self.header("Content-Type", content_type)
    }

    /// Set the user agent
    pub fn user_agent(mut self, user_agent: &str) -> Result<Self> {
        self.header("User-Agent", user_agent)
    }

    /// Set the authorization header
    pub fn authorization(mut self, auth: &str) -> Result<Self> {
        self.header("Authorization", auth)
    }

    /// Set basic authentication
    pub fn basic_auth(mut self, username: &str, password: Option<&str>) -> Self {
        self.reqwest_builder = self.reqwest_builder.basic_auth(username, password);
        self
    }

    /// Set bearer token authentication
    pub fn bearer_auth(mut self, token: &str) -> Result<Self> {
        self.authorization(&format!("Bearer {}", token))
    }

    /// Set the accept header
    pub fn accept(mut self, accept: &str) -> Result<Self> {
        self.header("Accept", accept)
    }

    /// Set JSON body
    pub fn json<T>(mut self, body: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        self.reqwest_builder = self.reqwest_builder.json(body);
        Ok(self)
    }

    /// Set text body
    pub fn text(mut self, body: &str) -> Result<Self> {
        self.reqwest_builder = self.reqwest_builder.body(body.to_string());
        Ok(self)
    }

    /// Set bytes body
    pub fn bytes(mut self, body: Vec<u8>) -> Result<Self> {
        self.reqwest_builder = self.reqwest_builder.body(body);
        Ok(self)
    }

    /// Set form data
    pub fn form<T>(mut self, data: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        self.reqwest_builder = self.reqwest_builder.form(data);
        Ok(self)
    }

    /// Set multipart form data
    pub fn multipart(mut self, form: reqwest::multipart::Form) -> Result<Self> {
        self.reqwest_builder = self.reqwest_builder.multipart(form);
        Ok(self)
    }

    /// Set query parameters
    pub fn query<T>(mut self, query: &T) -> Result<Self>
    where
        T: serde::Serialize,
    {
        self.reqwest_builder = self.reqwest_builder.query(query);
        Ok(self)
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.timeout(timeout);
        self.reqwest_builder = self.reqwest_builder.timeout(timeout);
        self
    }

    /// Set connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.connect_timeout(timeout);
        // Note: reqwest::RequestBuilder doesn't have connect_timeout method
        // self.reqwest_builder = self.reqwest_builder.connect_timeout(timeout);
        self
    }

    /// Set read timeout
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.read_timeout(timeout);
        self
    }

    /// Set write timeout
    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_config = self.timeout_config.write_timeout(timeout);
        self
    }

    /// Set version
    pub fn version(mut self, version: http::Version) -> Self {
        self.reqwest_builder = self.reqwest_builder.version(version);
        self
    }

    /// Build the request
    pub fn build(self) -> Result<Request> {
        let reqwest_request = self.reqwest_builder
            .build()
            .map_err(Error::Network)?;

        let method = reqwest_request.method().clone();
        let url = reqwest_request.url().clone();
        let headers = reqwest_request.headers().clone();
        let body = reqwest_request.body()
            .and_then(|b| b.as_bytes())
            .map(|bytes| RequestBody::Bytes(bytes.to_vec()))
            .unwrap_or(RequestBody::Empty);

        Ok(Request {
            method,
            url,
            headers,
            body: Some(body),
            timeout_config: self.timeout_config,
        })
    }

    /// Send the request and return the response
    pub async fn send(self) -> Result<Response> {
        let reqwest_response = self.reqwest_builder
            .send()
            .await
            .map_err(Error::Network)?;

        Response::from_reqwest_response(reqwest_response, self.cookie_jar).await
    }

    /// Send the request and return JSON response
    pub async fn send_json<T>(self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.send().await?;
        response.json().await
    }

    /// Send the request and return text response
    pub async fn send_text(self) -> Result<String> {
        let response = self.send().await?;
        response.text().await
    }

    /// Send the request and return bytes response
    pub async fn send_bytes(self) -> Result<Vec<u8>> {
        let response = self.send().await?;
        response.bytes().await
    }
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

impl std::fmt::Debug for RequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestBody::Empty => write!(f, "Empty"),
            RequestBody::Text(text) => write!(f, "Text({})", text),
            RequestBody::Json(json) => write!(f, "Json({})", json),
            RequestBody::Bytes(bytes) => write!(f, "Bytes({} bytes)", bytes.len()),
            RequestBody::Form(data) => write!(f, "Form({} pairs)", data.len()),
            RequestBody::Multipart(parts) => write!(f, "Multipart({} parts)", parts.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let url = "https://httpbin.org/get".parse().unwrap();
        let request = Request::new(Method::GET, url);
        assert_eq!(request.method(), &Method::GET);
    }

    #[test]
    fn test_request_builder_creation() {
        let client = reqwest::Client::new();
        let cookie_jar = CookieJar::new();
        let url = "https://httpbin.org/get".parse().unwrap();
        
        let builder = RequestBuilder::new(
            Arc::new(client),
            Arc::new(cookie_jar),
            Method::GET,
            url,
            TimeoutConfig::default(),
            HeaderMap::new(),
        );
        
        assert_eq!(builder.method(), &Method::GET);
    }
} 