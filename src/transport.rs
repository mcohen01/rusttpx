use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, Request as ReqwestRequest, Response as ReqwestResponse};

use crate::error::{Error, Result};
use crate::timeout::TimeoutConfig;

/// Transport trait for HTTP operations
///
/// This trait abstracts the underlying transport layer, allowing for
/// different implementations (HTTP/1.1, HTTP/2, custom protocols, etc.)
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a request and return the response
    async fn send(&self, request: ReqwestRequest) -> Result<ReqwestResponse>;
    
    /// Get the transport name/type
    fn name(&self) -> &str;
    
    /// Check if the transport is available
    fn is_available(&self) -> bool;
}

/// Default HTTP transport implementation using reqwest
pub struct HttpTransport {
    client: Arc<ReqwestClient>,
    timeout_config: TimeoutConfig,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(client: Arc<ReqwestClient>, timeout_config: TimeoutConfig) -> Self {
        Self {
            client,
            timeout_config,
        }
    }
    
    /// Get the underlying reqwest client
    pub fn client(&self) -> &ReqwestClient {
        &self.client
    }
    
    /// Get the timeout configuration
    pub fn timeout_config(&self) -> &TimeoutConfig {
        &self.timeout_config
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&self, request: ReqwestRequest) -> Result<ReqwestResponse> {
        let timeout = self.timeout_config.get_timeout();
        
        if let Some(timeout) = timeout {
            tokio::time::timeout(timeout, self.client.execute(request))
                .await
                .map_err(|_| Error::timeout(timeout))?
                .map_err(Error::Network)
        } else {
            self.client.execute(request).await.map_err(Error::Network)
        }
    }
    
    fn name(&self) -> &str {
        "HTTP/1.1"
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

/// HTTP/2 transport implementation
pub struct Http2Transport {
    client: Arc<ReqwestClient>,
    timeout_config: TimeoutConfig,
}

impl Http2Transport {
    /// Create a new HTTP/2 transport
    pub fn new(client: Arc<ReqwestClient>, timeout_config: TimeoutConfig) -> Self {
        Self {
            client,
            timeout_config,
        }
    }
}

#[async_trait]
impl Transport for Http2Transport {
    async fn send(&self, request: ReqwestRequest) -> Result<ReqwestResponse> {
        let timeout = self.timeout_config.get_timeout();
        
        if let Some(timeout) = timeout {
            tokio::time::timeout(timeout, self.client.execute(request))
                .await
                .map_err(|_| Error::timeout(timeout))?
                .map_err(Error::Network)
        } else {
            self.client.execute(request).await.map_err(Error::Network)
        }
    }
    
    fn name(&self) -> &str {
        "HTTP/2"
    }
    
    fn is_available(&self) -> bool {
        true
    }
}

/// Transport manager for handling multiple transport types
pub struct TransportManager {
    transports: Vec<Box<dyn Transport>>,
    default_transport: usize,
}

impl TransportManager {
    /// Create a new transport manager
    pub fn new() -> Self {
        Self {
            transports: Vec::new(),
            default_transport: 0,
        }
    }
    
    /// Add a transport to the manager
    pub fn add_transport(&mut self, transport: Box<dyn Transport>) {
        self.transports.push(transport);
    }
    
    /// Set the default transport by index
    pub fn set_default_transport(&mut self, index: usize) -> Result<()> {
        if index >= self.transports.len() {
            return Err(Error::config("Invalid transport index"));
        }
        self.default_transport = index;
        Ok(())
    }
    
    /// Get the default transport
    pub fn default_transport(&self) -> Option<&dyn Transport> {
        self.transports.get(self.default_transport).map(|t| t.as_ref())
    }
    
    /// Get a transport by name
    pub fn get_transport(&self, name: &str) -> Option<&dyn Transport> {
        self.transports.iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }
    
    /// Get all available transports
    pub fn available_transports(&self) -> Vec<&dyn Transport> {
        self.transports.iter()
            .filter(|t| t.is_available())
            .map(|t| t.as_ref())
            .collect()
    }
    
    /// Send a request using the default transport
    pub async fn send(&self, request: ReqwestRequest) -> Result<ReqwestResponse> {
        if let Some(transport) = self.default_transport() {
            transport.send(request).await
        } else {
            Err(Error::config("No default transport available"))
        }
    }
    
    /// Send a request using a specific transport
    pub async fn send_with_transport(&self, request: ReqwestRequest, transport_name: &str) -> Result<ReqwestResponse> {
        if let Some(transport) = self.get_transport(transport_name) {
            transport.send(request).await
        } else {
            Err(Error::config(format!("Transport '{}' not found", transport_name)))
        }
    }
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Transport configuration
#[derive(Clone, Debug)]
pub struct TransportConfig {
    /// Enable HTTP/2 support
    pub http2_enabled: bool,
    /// Enable HTTP/1.1 support
    pub http1_enabled: bool,
    /// Connection pool size
    pub pool_size: usize,
    /// Keep-alive timeout
    pub keep_alive_timeout: Option<Duration>,
    /// TCP keep-alive
    pub tcp_keep_alive: Option<Duration>,
    /// TCP nodelay
    pub tcp_nodelay: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            http2_enabled: true,
            http1_enabled: true,
            pool_size: 100,
            keep_alive_timeout: Some(Duration::from_secs(90)),
            tcp_keep_alive: Some(Duration::from_secs(60)),
            tcp_nodelay: true,
        }
    }
}

/// Transport builder for creating transport configurations
pub struct TransportBuilder {
    config: TransportConfig,
}

impl TransportBuilder {
    /// Create a new transport builder
    pub fn new() -> Self {
        Self {
            config: TransportConfig::default(),
        }
    }
    
    /// Enable HTTP/2
    pub fn http2(mut self, enabled: bool) -> Self {
        self.config.http2_enabled = enabled;
        self
    }
    
    /// Enable HTTP/1.1
    pub fn http1(mut self, enabled: bool) -> Self {
        self.config.http1_enabled = enabled;
        self
    }
    
    /// Set connection pool size
    pub fn pool_size(mut self, size: usize) -> Self {
        self.config.pool_size = size;
        self
    }
    
    /// Set keep-alive timeout
    pub fn keep_alive_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.config.keep_alive_timeout = timeout;
        self
    }
    
    /// Set TCP keep-alive
    pub fn tcp_keep_alive(mut self, keep_alive: Option<Duration>) -> Self {
        self.config.tcp_keep_alive = keep_alive;
        self
    }
    
    /// Set TCP nodelay
    pub fn tcp_nodelay(mut self, nodelay: bool) -> Self {
        self.config.tcp_nodelay = nodelay;
        self
    }
    
    /// Build the transport configuration
    pub fn build(self) -> TransportConfig {
        self.config
    }
}

impl Default for TransportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    
    #[tokio::test]
    async fn test_http_transport() {
        let client = Arc::new(Client::new());
        let timeout_config = TimeoutConfig::default();
        let transport = HttpTransport::new(client, timeout_config);
        
        assert_eq!(transport.name(), "HTTP/1.1");
        assert!(transport.is_available());
    }
    
    #[tokio::test]
    async fn test_http2_transport() {
        let client = Arc::new(Client::new());
        let timeout_config = TimeoutConfig::default();
        let transport = Http2Transport::new(client, timeout_config);
        
        assert_eq!(transport.name(), "HTTP/2");
        assert!(transport.is_available());
    }
    
    #[test]
    fn test_transport_manager() {
        let mut manager = TransportManager::new();
        
        let client = Arc::new(Client::new());
        let timeout_config = TimeoutConfig::default();
        
        let http_transport = Box::new(HttpTransport::new(client.clone(), timeout_config.clone()));
        let http2_transport = Box::new(Http2Transport::new(client, timeout_config));
        
        manager.add_transport(http_transport);
        manager.add_transport(http2_transport);
        
        assert_eq!(manager.available_transports().len(), 2);
        assert!(manager.get_transport("HTTP/1.1").is_some());
        assert!(manager.get_transport("HTTP/2").is_some());
    }
    
    #[test]
    fn test_transport_config() {
        let config = TransportConfig::default();
        
        assert!(config.http2_enabled);
        assert!(config.http1_enabled);
        assert_eq!(config.pool_size, 100);
        assert!(config.tcp_nodelay);
    }
    
    #[test]
    fn test_transport_builder() {
        let config = TransportBuilder::new()
            .http2(false)
            .pool_size(50)
            .tcp_nodelay(false)
            .build();
        
        assert!(!config.http2_enabled);
        assert_eq!(config.pool_size, 50);
        assert!(!config.tcp_nodelay);
    }
} 