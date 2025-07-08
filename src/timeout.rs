use std::time::Duration;

/// Configuration for various timeout settings
///
/// This struct holds timeout configurations for different aspects
/// of HTTP requests and connections.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Overall request timeout
    pub timeout: Option<Duration>,
    /// Connection establishment timeout
    pub connect_timeout: Option<Duration>,
    /// Read timeout
    pub read_timeout: Option<Duration>,
    /// Write timeout
    pub write_timeout: Option<Duration>,
    /// Pool idle timeout
    pub pool_idle_timeout: Option<Duration>,
}

impl TimeoutConfig {
    /// Create a new timeout configuration with a default timeout
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            connect_timeout: None,
            read_timeout: None,
            write_timeout: None,
            pool_idle_timeout: None,
        }
    }

    /// Set the overall request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Set the read timeout
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Set the write timeout
    pub fn write_timeout(mut self, timeout: Duration) -> Self {
        self.write_timeout = Some(timeout);
        self
    }

    /// Set the pool idle timeout
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = Some(timeout);
        self
    }

    /// Get the overall request timeout
    pub fn get_timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Get the connection timeout
    pub fn get_connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// Get the read timeout
    pub fn get_read_timeout(&self) -> Option<Duration> {
        self.read_timeout
    }

    /// Get the write timeout
    pub fn get_write_timeout(&self) -> Option<Duration> {
        self.write_timeout
    }

    /// Get the pool idle timeout
    pub fn get_pool_idle_timeout(&self) -> Option<Duration> {
        self.pool_idle_timeout
    }

    /// Check if any timeout is configured
    pub fn has_timeout(&self) -> bool {
        self.timeout.is_some()
            || self.connect_timeout.is_some()
            || self.read_timeout.is_some()
            || self.write_timeout.is_some()
            || self.pool_idle_timeout.is_some()
    }

    /// Get the effective timeout (overall timeout or sum of connect + read)
    pub fn get_effective_timeout(&self) -> Option<Duration> {
        if let Some(timeout) = self.timeout {
            return Some(timeout);
        }

        let connect = self.connect_timeout.unwrap_or(Duration::from_secs(30));
        let read = self.read_timeout.unwrap_or(Duration::from_secs(30));
        Some(connect + read)
    }

    /// Merge with another timeout configuration
    pub fn merge(mut self, other: &TimeoutConfig) -> Self {
        if other.timeout.is_some() {
            self.timeout = other.timeout;
        }
        if other.connect_timeout.is_some() {
            self.connect_timeout = other.connect_timeout;
        }
        if other.read_timeout.is_some() {
            self.read_timeout = other.read_timeout;
        }
        if other.write_timeout.is_some() {
            self.write_timeout = other.write_timeout;
        }
        if other.pool_idle_timeout.is_some() {
            self.pool_idle_timeout = other.pool_idle_timeout;
        }
        self
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            connect_timeout: Some(Duration::from_secs(10)),
            read_timeout: None,
            write_timeout: None,
            pool_idle_timeout: Some(Duration::from_secs(90)),
        }
    }
}

/// Predefined timeout configurations
impl TimeoutConfig {
    /// Create a timeout configuration suitable for quick requests
    pub fn quick() -> Self {
        Self {
            timeout: Some(Duration::from_secs(5)),
            connect_timeout: Some(Duration::from_secs(2)),
            read_timeout: Some(Duration::from_secs(3)),
            write_timeout: Some(Duration::from_secs(3)),
            pool_idle_timeout: Some(Duration::from_secs(30)),
        }
    }

    /// Create a timeout configuration suitable for long-running requests
    pub fn long() -> Self {
        Self {
            timeout: Some(Duration::from_secs(300)), // 5 minutes
            connect_timeout: Some(Duration::from_secs(30)),
            read_timeout: Some(Duration::from_secs(270)),
            write_timeout: Some(Duration::from_secs(270)),
            pool_idle_timeout: Some(Duration::from_secs(300)),
        }
    }

    /// Create a timeout configuration with no timeouts (unlimited)
    pub fn unlimited() -> Self {
        Self {
            timeout: None,
            connect_timeout: None,
            read_timeout: None,
            write_timeout: None,
            pool_idle_timeout: None,
        }
    }

    /// Create a timeout configuration for streaming requests
    pub fn streaming() -> Self {
        Self {
            timeout: None, // No overall timeout for streaming
            connect_timeout: Some(Duration::from_secs(10)),
            read_timeout: Some(Duration::from_secs(60)), // 1 minute read timeout
            write_timeout: Some(Duration::from_secs(60)), // 1 minute write timeout
            pool_idle_timeout: Some(Duration::from_secs(90)),
        }
    }
}

/// Timeout error types
#[derive(Debug, thiserror::Error)]
pub enum TimeoutError {
    /// Request timed out
    #[error("Request timed out after {duration:?}")]
    RequestTimeout { duration: Duration },

    /// Connection timed out
    #[error("Connection timed out after {duration:?}")]
    ConnectionTimeout { duration: Duration },

    /// Read timed out
    #[error("Read timed out after {duration:?}")]
    ReadTimeout { duration: Duration },

    /// Write timed out
    #[error("Write timed out after {duration:?}")]
    WriteTimeout { duration: Duration },

    /// Pool idle timeout
    #[error("Pool idle timeout after {duration:?}")]
    PoolIdleTimeout { duration: Duration },
}

impl TimeoutError {
    /// Create a request timeout error
    pub fn request_timeout(duration: Duration) -> Self {
        TimeoutError::RequestTimeout { duration }
    }

    /// Create a connection timeout error
    pub fn connection_timeout(duration: Duration) -> Self {
        TimeoutError::ConnectionTimeout { duration }
    }

    /// Create a read timeout error
    pub fn read_timeout(duration: Duration) -> Self {
        TimeoutError::ReadTimeout { duration }
    }

    /// Create a write timeout error
    pub fn write_timeout(duration: Duration) -> Self {
        TimeoutError::WriteTimeout { duration }
    }

    /// Create a pool idle timeout error
    pub fn pool_idle_timeout(duration: Duration) -> Self {
        TimeoutError::PoolIdleTimeout { duration }
    }

    /// Get the duration associated with this timeout error
    pub fn duration(&self) -> Duration {
        match self {
            TimeoutError::RequestTimeout { duration } => *duration,
            TimeoutError::ConnectionTimeout { duration } => *duration,
            TimeoutError::ReadTimeout { duration } => *duration,
            TimeoutError::WriteTimeout { duration } => *duration,
            TimeoutError::PoolIdleTimeout { duration } => *duration,
        }
    }
}

/// Timeout utilities
pub mod utils {
    use super::*;
    use tokio::time::{timeout, Timeout};

    /// Apply a timeout to a future
    pub async fn with_timeout<F, T>(
        future: F,
        timeout_duration: Duration,
    ) -> Result<T, TimeoutError>
    where
        F: std::future::Future<Output = T>,
    {
        match timeout(timeout_duration, future).await {
            Ok(result) => Ok(result),
            Err(_) => Err(TimeoutError::request_timeout(timeout_duration)),
        }
    }

    /// Apply a timeout to a future and return a timeout future
    pub fn with_timeout_future<F, T>(
        future: F,
        timeout_duration: Duration,
    ) -> Timeout<F>
    where
        F: std::future::Future<Output = T>,
    {
        timeout(timeout_duration, future)
    }

    /// Check if a duration is reasonable for a timeout
    pub fn is_reasonable_timeout(duration: Duration) -> bool {
        duration > Duration::from_millis(0) && duration < Duration::from_secs(3600) // 1 hour max
    }

    /// Get a reasonable timeout duration based on the request type
    pub fn get_reasonable_timeout(method: &http::Method, has_body: bool) -> Duration {
        match method.as_str() {
            "GET" | "HEAD" => {
                if has_body {
                    Duration::from_secs(60)
                } else {
                    Duration::from_secs(30)
                }
            }
            "POST" | "PUT" | "PATCH" => Duration::from_secs(120),
            "DELETE" => Duration::from_secs(60),
            _ => Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_creation() {
        let config = TimeoutConfig::new(Duration::from_secs(30));
        assert_eq!(config.get_timeout(), Some(Duration::from_secs(30)));
        assert!(config.has_timeout());
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert!(config.has_timeout());
        assert_eq!(config.get_timeout(), Some(Duration::from_secs(30)));
        assert_eq!(config.get_connect_timeout(), Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_timeout_config_quick() {
        let config = TimeoutConfig::quick();
        assert_eq!(config.get_timeout(), Some(Duration::from_secs(5)));
        assert_eq!(config.get_connect_timeout(), Some(Duration::from_secs(2)));
    }

    #[test]
    fn test_timeout_config_long() {
        let config = TimeoutConfig::long();
        assert_eq!(config.get_timeout(), Some(Duration::from_secs(300)));
        assert_eq!(config.get_connect_timeout(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_timeout_config_unlimited() {
        let config = TimeoutConfig::unlimited();
        assert!(!config.has_timeout());
        assert_eq!(config.get_timeout(), None);
    }

    #[test]
    fn test_timeout_config_merge() {
        let mut config1 = TimeoutConfig::new(Duration::from_secs(30));
        let config2 = TimeoutConfig::new(Duration::from_secs(60));
        
        let merged = config1.merge(&config2);
        assert_eq!(merged.get_timeout(), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_timeout_error() {
        let error = TimeoutError::request_timeout(Duration::from_secs(30));
        assert_eq!(error.duration(), Duration::from_secs(30));
    }

    #[test]
    fn test_utils() {
        assert!(utils::is_reasonable_timeout(Duration::from_secs(30)));
        assert!(!utils::is_reasonable_timeout(Duration::from_millis(0)));
        assert!(!utils::is_reasonable_timeout(Duration::from_secs(7200))); // 2 hours

        let timeout = utils::get_reasonable_timeout(&http::Method::GET, false);
        assert_eq!(timeout, Duration::from_secs(30));

        let timeout = utils::get_reasonable_timeout(&http::Method::POST, true);
        assert_eq!(timeout, Duration::from_secs(120));
    }
} 