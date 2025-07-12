use thiserror::Error;

/// Result type for RustTPX operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for RustTPX
#[derive(Error, Debug)]
pub enum Error {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// URL parsing errors
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// HTTP protocol errors
    #[error("HTTP error: {0}")]
    Http(#[from] http::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Timeout errors
    #[error("Request timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },

    /// SSL/TLS errors
    #[error("SSL/TLS error: {0}")]
    Tls(String),

    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Proxy errors
    #[error("Proxy error: {0}")]
    Proxy(String),

    /// Compression errors
    #[error("Compression error: {0}")]
    Compression(String),

    /// Multipart form data errors
    #[error("Multipart error: {0}")]
    Multipart(String),

    /// Cookie errors
    #[error("Cookie error: {0}")]
    Cookie(String),

    /// Invalid request configuration
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Response parsing errors
    #[error("Response parsing error: {0}")]
    ResponseParse(String),

    /// Stream errors
    #[error("Stream error: {0}")]
    Stream(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),

    /// Wrapper for other error types
    #[error("Other error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    /// Create a new timeout error
    pub fn timeout(duration: std::time::Duration) -> Self {
        Error::Timeout { duration }
    }

    /// Create a new TLS error
    pub fn tls(message: impl Into<String>) -> Self {
        Error::Tls(message.into())
    }

    /// Create a new authentication error
    pub fn auth(message: impl Into<String>) -> Self {
        Error::Auth(message.into())
    }

    /// Create a new proxy error
    pub fn proxy(message: impl Into<String>) -> Self {
        Error::Proxy(message.into())
    }

    /// Create a new compression error
    pub fn compression(message: impl Into<String>) -> Self {
        Error::Compression(message.into())
    }

    /// Create a new multipart error
    pub fn multipart(message: impl Into<String>) -> Self {
        Error::Multipart(message.into())
    }

    /// Create a new cookie error
    pub fn cookie(message: impl Into<String>) -> Self {
        Error::Cookie(message.into())
    }

    /// Create a new invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Error::InvalidRequest(message.into())
    }

    /// Create a new response parsing error
    pub fn response_parse(message: impl Into<String>) -> Self {
        Error::ResponseParse(message.into())
    }

    /// Create a new stream error
    pub fn stream(message: impl Into<String>) -> Self {
        Error::Stream(message.into())
    }

    /// Create a new configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Error::Config(message.into())
    }

    /// Create a new custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Error::Custom(message.into())
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Error::Timeout { .. })
    }

    /// Check if this is a network error
    pub fn is_network(&self) -> bool {
        matches!(self, Error::Network(_))
    }

    /// Check if this is a TLS error
    pub fn is_tls(&self) -> bool {
        matches!(self, Error::Tls(_))
    }

    /// Check if this is an authentication error
    pub fn is_auth(&self) -> bool {
        matches!(self, Error::Auth(_))
    }

    /// Get the underlying reqwest error if this is a network error
    pub fn as_network_error(&self) -> Option<&reqwest::Error> {
        match self {
            Error::Network(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Custom(format!("IO error: {}", err))
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Error::Timeout {
            duration: std::time::Duration::from_secs(0), // We don't have the original duration
        }
    }
}

impl From<http::header::InvalidHeaderName> for Error {
    fn from(err: http::header::InvalidHeaderName) -> Self {
        Error::InvalidRequest(format!("Invalid header name: {}", err))
    }
}

impl From<http::header::InvalidHeaderValue> for Error {
    fn from(err: http::header::InvalidHeaderValue) -> Self {
        Error::InvalidRequest(format!("Invalid header value: {}", err))
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_string())
    }
}

/// HTTP status code error
#[derive(Error, Debug)]
pub enum StatusError {
    /// Client error (4xx status codes)
    #[error("Client error: {status} - {message}")]
    Client { status: http::StatusCode, message: String },

    /// Server error (5xx status codes)
    #[error("Server error: {status} - {message}")]
    Server { status: http::StatusCode, message: String },

    /// Unexpected status code
    #[error("Unexpected status: {status} - {message}")]
    Unexpected { status: http::StatusCode, message: String },
}

impl StatusError {
    /// Create a new client error
    pub fn client(status: http::StatusCode, message: impl Into<String>) -> Self {
        StatusError::Client {
            status,
            message: message.into(),
        }
    }

    /// Create a new server error
    pub fn server(status: http::StatusCode, message: impl Into<String>) -> Self {
        StatusError::Server {
            status,
            message: message.into(),
        }
    }

    /// Create a new unexpected status error
    pub fn unexpected(status: http::StatusCode, message: impl Into<String>) -> Self {
        StatusError::Unexpected {
            status,
            message: message.into(),
        }
    }

    /// Get the status code
    pub fn status(&self) -> http::StatusCode {
        match self {
            StatusError::Client { status, .. } => *status,
            StatusError::Server { status, .. } => *status,
            StatusError::Unexpected { status, .. } => *status,
        }
    }

    /// Check if this is a client error
    pub fn is_client_error(&self) -> bool {
        matches!(self, StatusError::Client { .. })
    }

    /// Check if this is a server error
    pub fn is_server_error(&self) -> bool {
        matches!(self, StatusError::Server { .. })
    }
}

impl From<StatusError> for Error {
    fn from(err: StatusError) -> Self {
        Error::Custom(err.to_string())
    }
} 