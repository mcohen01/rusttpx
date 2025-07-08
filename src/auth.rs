use std::collections::HashMap;
use http::{HeaderMap, HeaderValue};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::error::{Error, Result};

/// Authentication configuration for HTTP requests
///
/// This struct holds various authentication methods and credentials
/// that can be used for HTTP requests.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,
    /// Additional headers for authentication
    pub headers: HeaderMap,
    /// Custom authentication data
    pub custom_data: HashMap<String, String>,
}

/// Authentication types
#[derive(Debug, Clone)]
pub enum AuthType {
    /// No authentication
    None,
    /// Basic authentication
    Basic {
        username: String,
        password: String,
    },
    /// Bearer token authentication
    Bearer {
        token: String,
    },
    /// API key authentication
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
    /// Digest authentication
    Digest {
        username: String,
        password: String,
        realm: Option<String>,
    },
    /// OAuth2 authentication
    OAuth2 {
        access_token: String,
        token_type: Option<String>,
    },
    /// Custom authentication
    Custom {
        scheme: String,
        credentials: String,
    },
}

/// API key location
#[derive(Debug, Clone)]
pub enum ApiKeyLocation {
    /// In the Authorization header
    Header,
    /// In the query parameters
    Query,
    /// In the request body
    Body,
}

impl ApiKeyLocation {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiKeyLocation::Header => "header",
            ApiKeyLocation::Query => "query",
            ApiKeyLocation::Body => "body",
        }
    }
}

impl AuthConfig {
    /// Create a new authentication configuration
    pub fn new() -> Self {
        Self {
            auth_type: AuthType::None,
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Create a basic authentication configuration
    pub fn basic(username: &str, password: &str) -> Self {
        Self {
            auth_type: AuthType::Basic {
                username: username.to_string(),
                password: password.to_string(),
            },
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Create a bearer token authentication configuration
    pub fn bearer(token: &str) -> Self {
        Self {
            auth_type: AuthType::Bearer {
                token: token.to_string(),
            },
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Create an API key authentication configuration
    pub fn api_key(key: &str, value: &str, location: ApiKeyLocation) -> Self {
        Self {
            auth_type: AuthType::ApiKey {
                key: key.to_string(),
                value: value.to_string(),
                location,
            },
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Create a digest authentication configuration
    pub fn digest(username: &str, password: &str, realm: Option<&str>) -> Self {
        Self {
            auth_type: AuthType::Digest {
                username: username.to_string(),
                password: password.to_string(),
                realm: realm.map(|s| s.to_string()),
            },
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Create an OAuth2 authentication configuration
    pub fn oauth2(access_token: &str, token_type: Option<&str>) -> Self {
        Self {
            auth_type: AuthType::OAuth2 {
                access_token: access_token.to_string(),
                token_type: token_type.map(|s| s.to_string()),
            },
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Create a custom authentication configuration
    pub fn custom(scheme: &str, credentials: &str) -> Self {
        Self {
            auth_type: AuthType::Custom {
                scheme: scheme.to_string(),
                credentials: credentials.to_string(),
            },
            headers: HeaderMap::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Set the authentication type
    pub fn auth_type(mut self, auth_type: AuthType) -> Self {
        self.auth_type = auth_type;
        self
    }

    /// Add a custom header
    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        let name = name.parse::<http::header::HeaderName>()?;
        let value = value.parse::<HeaderValue>()?;
        self.headers.insert(name, value);
        Ok(self)
    }

    /// Add custom data
    pub fn custom_data(mut self, key: &str, value: &str) -> Self {
        self.custom_data.insert(key.to_string(), value.to_string());
        self
    }

    /// Get the authentication type
    pub fn get_auth_type(&self) -> &AuthType {
        &self.auth_type
    }

    /// Get the headers
    pub fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get custom data
    pub fn get_custom_data(&self) -> &HashMap<String, String> {
        &self.custom_data
    }

    /// Check if authentication is configured
    pub fn has_auth(&self) -> bool {
        !matches!(self.auth_type, AuthType::None)
    }

    /// Get the authorization header value
    pub fn get_authorization_header(&self) -> Option<String> {
        match &self.auth_type {
            AuthType::None => None,
            AuthType::Basic { username, password } => {
                let credentials = format!("{}:{}", username, password);
                let encoded = BASE64.encode(credentials.as_bytes());
                Some(format!("Basic {}", encoded))
            }
            AuthType::Bearer { token } => {
                Some(format!("Bearer {}", token))
            }
            AuthType::ApiKey { key, value, location } => {
                match location {
                    ApiKeyLocation::Header => Some(format!("{} {}", key, value)),
                    _ => None,
                }
            }
            AuthType::Digest { .. } => {
                // Digest auth requires challenge-response, so we can't generate a static header
                None
            }
            AuthType::OAuth2 { access_token, token_type } => {
                let token_type = token_type.as_deref().unwrap_or("Bearer");
                Some(format!("{} {}", token_type, access_token))
            }
            AuthType::Custom { scheme, credentials } => {
                Some(format!("{} {}", scheme, credentials))
            }
        }
    }

    /// Get query parameters for API key authentication
    pub fn get_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let AuthType::ApiKey { key, value, location } = &self.auth_type {
            if matches!(location, ApiKeyLocation::Query) {
                params.insert(key.clone(), value.clone());
            }
        }
        
        params
    }

    /// Get body parameters for API key authentication
    pub fn get_body_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        if let AuthType::ApiKey { key, value, location } = &self.auth_type {
            if matches!(location, ApiKeyLocation::Body) {
                params.insert(key.clone(), value.clone());
            }
        }
        
        params
    }

    /// Apply authentication to headers
    pub fn apply_to_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        // Add custom headers
        for (name, value) in &self.headers {
            headers.insert(name, value.clone());
        }

        // Add authorization header if available
        if let Some(auth_header) = self.get_authorization_header() {
            let value = auth_header.parse::<HeaderValue>()?;
            headers.insert("Authorization", value);
        }

        Ok(())
    }

    /// Merge with another authentication configuration
    pub fn merge(mut self, other: &AuthConfig) -> Self {
        // Merge headers
        for (name, value) in &other.headers {
            self.headers.insert(name, value.clone());
        }

        // Merge custom data
        for (key, value) in &other.custom_data {
            self.custom_data.insert(key.clone(), value.clone());
        }

        // Use other's auth type if it's not None
        if matches!(other.auth_type, AuthType::None) {
            // Keep current auth type
        } else {
            self.auth_type = other.auth_type.clone();
        }

        self
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication builder
pub struct AuthBuilder {
    config: AuthConfig,
}

impl AuthBuilder {
    /// Create a new authentication builder
    pub fn new() -> Self {
        Self {
            config: AuthConfig::new(),
        }
    }

    /// Set basic authentication
    pub fn basic(mut self, username: &str, password: &str) -> Self {
        self.config = self.config.auth_type(AuthType::Basic {
            username: username.to_string(),
            password: password.to_string(),
        });
        self
    }

    /// Set bearer token authentication
    pub fn bearer(mut self, token: &str) -> Self {
        self.config = self.config.auth_type(AuthType::Bearer {
            token: token.to_string(),
        });
        self
    }

    /// Set API key authentication in header
    pub fn api_key_header(mut self, key: &str, value: &str) -> Self {
        self.config = self.config.auth_type(AuthType::ApiKey {
            key: key.to_string(),
            value: value.to_string(),
            location: ApiKeyLocation::Header,
        });
        self
    }

    /// Set API key authentication in query
    pub fn api_key_query(mut self, key: &str, value: &str) -> Self {
        self.config = self.config.auth_type(AuthType::ApiKey {
            key: key.to_string(),
            value: value.to_string(),
            location: ApiKeyLocation::Query,
        });
        self
    }

    /// Set API key authentication in body
    pub fn api_key_body(mut self, key: &str, value: &str) -> Self {
        self.config = self.config.auth_type(AuthType::ApiKey {
            key: key.to_string(),
            value: value.to_string(),
            location: ApiKeyLocation::Body,
        });
        self
    }

    /// Set OAuth2 authentication
    pub fn oauth2(mut self, access_token: &str, token_type: Option<&str>) -> Self {
        self.config = self.config.auth_type(AuthType::OAuth2 {
            access_token: access_token.to_string(),
            token_type: token_type.map(|s| s.to_string()),
        });
        self
    }

    /// Set custom authentication
    pub fn custom(mut self, scheme: &str, credentials: &str) -> Self {
        self.config = self.config.auth_type(AuthType::Custom {
            scheme: scheme.to_string(),
            credentials: credentials.to_string(),
        });
        self
    }

    /// Add a custom header
    pub fn header(mut self, name: &str, value: &str) -> Result<Self> {
        self.config = self.config.header(name, value)?;
        Ok(self)
    }

    /// Add custom data
    pub fn custom_data(mut self, key: &str, value: &str) -> Self {
        self.config = self.config.custom_data(key, value);
        self
    }

    /// Build the authentication configuration
    pub fn build(self) -> AuthConfig {
        self.config
    }
}

impl Default for AuthBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods for authentication operations
impl AuthConfig {
    /// Create an authentication builder
    pub fn builder() -> AuthBuilder {
        AuthBuilder::new()
    }

    /// Create authentication from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::new();

        // Check for basic auth
        if let (Ok(username), Ok(password)) = (std::env::var("HTTP_USERNAME"), std::env::var("HTTP_PASSWORD")) {
            config = config.auth_type(AuthType::Basic { username, password });
        }

        // Check for bearer token
        if let Ok(token) = std::env::var("HTTP_BEARER_TOKEN") {
            config = config.auth_type(AuthType::Bearer { token });
        }

        // Check for API key
        if let (Ok(key), Ok(value)) = (std::env::var("HTTP_API_KEY"), std::env::var("HTTP_API_VALUE")) {
            let location = std::env::var("HTTP_API_LOCATION")
                .map(|loc| match loc.as_str() {
                    "query" => ApiKeyLocation::Query,
                    "body" => ApiKeyLocation::Body,
                    _ => ApiKeyLocation::Header,
                })
                .unwrap_or(ApiKeyLocation::Header);
            
            config = config.auth_type(AuthType::ApiKey { key, value, location });
        }

        config
    }

    /// Create authentication for GitHub API
    pub fn github(token: &str) -> Self {
        Self::bearer(token)
    }

    /// Create authentication for AWS
    pub fn aws(access_key: &str, secret_key: &str) -> Self {
        // AWS uses a complex signing process, this is a simplified version
        Self::custom("AWS4-HMAC-SHA256", &format!("{}:{}", access_key, secret_key))
    }

    /// Create authentication for Google Cloud
    pub fn google_cloud(access_token: &str) -> Self {
        Self::oauth2(access_token, Some("Bearer"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_creation() {
        let config = AuthConfig::new();
        assert!(!config.has_auth());
    }

    #[test]
    fn test_basic_auth() {
        let config = AuthConfig::basic("user", "pass");
        assert!(config.has_auth());
        
        let auth_header = config.get_authorization_header();
        assert!(auth_header.is_some());
        assert!(auth_header.unwrap().starts_with("Basic "));
    }

    #[test]
    fn test_bearer_auth() {
        let config = AuthConfig::bearer("token123");
        assert!(config.has_auth());
        
        let auth_header = config.get_authorization_header();
        assert_eq!(auth_header, Some("Bearer token123".to_string()));
    }

    #[test]
    fn test_api_key_auth() {
        let config = AuthConfig::api_key("X-API-Key", "secret", ApiKeyLocation::Header);
        assert!(config.has_auth());
        
        let auth_header = config.get_authorization_header();
        assert_eq!(auth_header, Some("X-API-Key secret".to_string()));
    }

    #[test]
    fn test_oauth2_auth() {
        let config = AuthConfig::oauth2("token123", Some("Bearer"));
        assert!(config.has_auth());
        
        let auth_header = config.get_authorization_header();
        assert_eq!(auth_header, Some("Bearer token123".to_string()));
    }

    #[test]
    fn test_auth_builder() {
        let config = AuthBuilder::new()
            .basic("user", "pass")
            .header("X-Custom", "value")
            .unwrap()
            .build();
        
        assert!(config.has_auth());
        assert_eq!(config.get_headers().len(), 1);
    }

    #[test]
    fn test_api_key_location() {
        assert_eq!(ApiKeyLocation::Header.as_str(), "header");
        assert_eq!(ApiKeyLocation::Query.as_str(), "query");
        assert_eq!(ApiKeyLocation::Body.as_str(), "body");
    }
} 