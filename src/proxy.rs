use std::collections::HashMap;
use reqwest::{ClientBuilder as ReqwestBuilder, Proxy as ReqwestProxy};
use url::Url;

use crate::error::{Error, Result};

/// Proxy configuration for HTTP requests
///
/// This struct holds configuration for HTTP proxies, including
/// authentication and different proxy types.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// HTTP proxy URL
    pub http_proxy: Option<Url>,
    /// HTTPS proxy URL
    pub https_proxy: Option<Url>,
    /// Proxy authentication
    pub auth: Option<ProxyAuth>,
    /// Proxy bypass patterns
    pub bypass: Vec<String>,
    /// Custom proxy for specific hosts
    pub custom_proxies: HashMap<String, Url>,
}

/// Proxy authentication
#[derive(Debug, Clone)]
pub struct ProxyAuth {
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

impl ProxyAuth {
    /// Create a new proxy authentication
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get the password
    pub fn password(&self) -> &str {
        &self.password
    }
}

impl ProxyConfig {
    /// Create a new proxy configuration
    pub fn new() -> Self {
        Self {
            http_proxy: None,
            https_proxy: None,
            auth: None,
            bypass: Vec::new(),
            custom_proxies: HashMap::new(),
        }
    }

    /// Set HTTP proxy
    pub fn http_proxy(mut self, url: Url) -> Self {
        self.http_proxy = Some(url);
        self
    }

    /// Set HTTPS proxy
    pub fn https_proxy(mut self, url: Url) -> Self {
        self.https_proxy = Some(url);
        self
    }

    /// Set both HTTP and HTTPS proxies to the same URL
    pub fn proxy(mut self, url: Url) -> Self {
        self.http_proxy = Some(url.clone());
        self.https_proxy = Some(url);
        self
    }

    /// Set proxy authentication
    pub fn auth(mut self, auth: ProxyAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set proxy authentication with username and password
    pub fn auth_credentials(mut self, username: &str, password: &str) -> Self {
        self.auth = Some(ProxyAuth::new(username, password));
        self
    }

    /// Add a bypass pattern
    pub fn bypass(mut self, pattern: &str) -> Self {
        self.bypass.push(pattern.to_string());
        self
    }

    /// Add multiple bypass patterns
    pub fn bypass_patterns(mut self, patterns: Vec<String>) -> Self {
        self.bypass.extend(patterns);
        self
    }

    /// Add a custom proxy for a specific host
    pub fn custom_proxy(mut self, host: &str, url: Url) -> Self {
        self.custom_proxies.insert(host.to_string(), url);
        self
    }

    /// Get the HTTP proxy URL
    pub fn get_http_proxy(&self) -> Option<&Url> {
        self.http_proxy.as_ref()
    }

    /// Get the HTTPS proxy URL
    pub fn get_https_proxy(&self) -> Option<&Url> {
        self.https_proxy.as_ref()
    }

    /// Get the proxy authentication
    pub fn get_auth(&self) -> Option<&ProxyAuth> {
        self.auth.as_ref()
    }

    /// Get the bypass patterns
    pub fn get_bypass(&self) -> &[String] {
        &self.bypass
    }

    /// Get custom proxies
    pub fn get_custom_proxies(&self) -> &HashMap<String, Url> {
        &self.custom_proxies
    }

    /// Check if any proxy is configured
    pub fn has_proxy(&self) -> bool {
        self.http_proxy.is_some()
            || self.https_proxy.is_some()
            || !self.custom_proxies.is_empty()
    }

    /// Check if a URL should bypass the proxy
    pub fn should_bypass(&self, url: &Url) -> bool {
        let host = url.host_str().unwrap_or("");
        
        for pattern in &self.bypass {
            if host.contains(pattern) || url.as_str().contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// Get the appropriate proxy for a URL
    pub fn get_proxy_for_url(&self, url: &Url) -> Option<&Url> {
        // Check custom proxies first
        if let Some(host) = url.host_str() {
            if let Some(proxy) = self.custom_proxies.get(host) {
                return Some(proxy);
            }
        }

        // Check scheme-based proxies
        match url.scheme() {
            "http" => self.http_proxy.as_ref(),
            "https" => self.https_proxy.as_ref(),
            _ => None,
        }
    }

    /// Apply this configuration to a reqwest client builder
    pub fn apply_to_builder(self, mut builder: ReqwestBuilder) -> ReqwestBuilder {
        // Apply HTTP proxy
        if let Some(http_proxy) = self.http_proxy {
            if let Ok(proxy) = ReqwestProxy::http(http_proxy.as_str()) {
                builder = builder.proxy(proxy);
            }
        }

        // Apply HTTPS proxy
        if let Some(https_proxy) = self.https_proxy {
            if let Ok(proxy) = ReqwestProxy::https(https_proxy.as_str()) {
                builder = builder.proxy(proxy);
            }
        }

        // Apply authentication if available
        if let Some(auth) = self.auth {
            // Note: Reqwest handles proxy auth automatically from the URL
            // This is a placeholder for future implementation
        }

        builder
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Proxy types
#[derive(Debug, Clone)]
pub enum ProxyType {
    /// HTTP proxy
    Http,
    /// HTTPS proxy
    Https,
    /// SOCKS4 proxy
    Socks4,
    /// SOCKS5 proxy
    Socks5,
}

impl ProxyType {
    /// Get the scheme for this proxy type
    pub fn scheme(&self) -> &'static str {
        match self {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks4 => "socks4",
            ProxyType::Socks5 => "socks5",
        }
    }
}

/// Proxy builder for creating proxy configurations
pub struct ProxyBuilder {
    config: ProxyConfig,
}

impl ProxyBuilder {
    /// Create a new proxy builder
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::new(),
        }
    }

    /// Set HTTP proxy
    pub fn http(mut self, url: &str) -> Result<Self> {
        let url = url.parse::<Url>()
            .map_err(|e| Error::proxy(format!("Invalid HTTP proxy URL: {}", e)))?;
        self.config = self.config.http_proxy(url);
        Ok(self)
    }

    /// Set HTTPS proxy
    pub fn https(mut self, url: &str) -> Result<Self> {
        let url = url.parse::<Url>()
            .map_err(|e| Error::proxy(format!("Invalid HTTPS proxy URL: {}", e)))?;
        self.config = self.config.https_proxy(url);
        Ok(self)
    }

    /// Set both HTTP and HTTPS proxies
    pub fn all(mut self, url: &str) -> Result<Self> {
        let url = url.parse::<Url>()
            .map_err(|e| Error::proxy(format!("Invalid proxy URL: {}", e)))?;
        self.config = self.config.proxy(url);
        Ok(self)
    }

    /// Set proxy with authentication
    pub fn with_auth(mut self, url: &str, username: &str, password: &str) -> Result<Self> {
        let mut url = url.parse::<Url>()
            .map_err(|e| Error::proxy(format!("Invalid proxy URL: {}", e)))?;
        
        // Add authentication to URL
        url.set_username(username)
            .map_err(|_| Error::proxy("Invalid username".to_string()))?;
        url.set_password(Some(password))
            .map_err(|_| Error::proxy("Invalid password".to_string()))?;
        
        self.config = self.config.proxy(url);
        Ok(self)
    }

    /// Add bypass pattern
    pub fn bypass(mut self, pattern: &str) -> Self {
        self.config = self.config.bypass(pattern);
        self
    }

    /// Add custom proxy for specific host
    pub fn custom(mut self, host: &str, url: &str) -> Result<Self> {
        let url = url.parse::<Url>()
            .map_err(|e| Error::proxy(format!("Invalid custom proxy URL: {}", e)))?;
        self.config = self.config.custom_proxy(host, url);
        Ok(self)
    }

    /// Build the proxy configuration
    pub fn build(self) -> ProxyConfig {
        self.config
    }
}

impl Default for ProxyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods for proxy operations
impl ProxyConfig {
    /// Create a proxy builder
    pub fn builder() -> ProxyBuilder {
        ProxyBuilder::new()
    }

    /// Create a proxy configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::new();

        // Check for HTTP_PROXY
        if let Ok(http_proxy) = std::env::var("HTTP_PROXY") {
            if let Ok(url) = http_proxy.parse::<Url>() {
                config = config.http_proxy(url);
            }
        }

        // Check for HTTPS_PROXY
        if let Ok(https_proxy) = std::env::var("HTTPS_PROXY") {
            if let Ok(url) = https_proxy.parse::<Url>() {
                config = config.https_proxy(url);
            }
        }

        // Check for NO_PROXY
        if let Ok(no_proxy) = std::env::var("NO_PROXY") {
            let bypass_patterns: Vec<String> = no_proxy
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            config = config.bypass_patterns(bypass_patterns);
        }

        config
    }

    /// Create a proxy configuration for localhost
    pub fn localhost(port: u16) -> Self {
        let url = format!("http://localhost:{}", port)
            .parse::<Url>()
            .expect("Invalid localhost URL");
        Self::new().proxy(url)
    }

    /// Create a proxy configuration for a specific host and port
    pub fn host_port(host: &str, port: u16) -> Result<Self> {
        let url = format!("http://{}:{}", host, port)
            .parse::<Url>()
            .map_err(|e| Error::proxy(format!("Invalid host:port URL: {}", e)))?;
        Ok(Self::new().proxy(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_creation() {
        let config = ProxyConfig::new();
        assert!(!config.has_proxy());
    }

    #[test]
    fn test_proxy_config_with_proxy() {
        let url = "http://proxy.example.com:8080".parse().unwrap();
        let config = ProxyConfig::new().proxy(url.clone());
        
        assert!(config.has_proxy());
        assert_eq!(config.get_http_proxy(), Some(&url));
        assert_eq!(config.get_https_proxy(), Some(&url));
    }

    #[test]
    fn test_proxy_auth() {
        let auth = ProxyAuth::new("user", "pass");
        assert_eq!(auth.username(), "user");
        assert_eq!(auth.password(), "pass");
    }

    #[test]
    fn test_proxy_builder() {
        let config = ProxyBuilder::new()
            .http("http://proxy.example.com:8080")
            .unwrap()
            .bypass("localhost")
            .build();
        
        assert!(config.has_proxy());
        assert_eq!(config.get_bypass().len(), 1);
    }

    #[test]
    fn test_proxy_bypass() {
        let config = ProxyConfig::new()
            .proxy("http://proxy.example.com:8080".parse().unwrap())
            .bypass("localhost");
        
        let localhost_url = "http://localhost:3000".parse().unwrap();
        assert!(config.should_bypass(&localhost_url));
        
        let external_url = "http://example.com".parse().unwrap();
        assert!(!config.should_bypass(&external_url));
    }

    #[test]
    fn test_proxy_type() {
        assert_eq!(ProxyType::Http.scheme(), "http");
        assert_eq!(ProxyType::Https.scheme(), "https");
        assert_eq!(ProxyType::Socks4.scheme(), "socks4");
        assert_eq!(ProxyType::Socks5.scheme(), "socks5");
    }
} 