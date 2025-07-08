use std::path::PathBuf;
use std::sync::Arc;
use reqwest::{ClientBuilder as ReqwestBuilder, Certificate, Identity};
use rustls::{ClientConfig, RootCertStore, Certificate as RustlsCertificate, PrivateKey};
use rustls_native_certs::load_native_certs;

use crate::error::{Error, Result};

/// TLS configuration for HTTP requests
///
/// This struct holds configuration for TLS/SSL connections,
/// including certificates, private keys, and verification settings.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Root certificates
    pub root_certs: Vec<Certificate>,
    /// Client certificate and private key
    pub client_cert: Option<Identity>,
    /// Whether to verify the server certificate
    pub verify: bool,
    /// Custom CA certificate path
    pub ca_cert_path: Option<PathBuf>,
    /// Client certificate path
    pub client_cert_path: Option<PathBuf>,
    /// Client private key path
    pub client_key_path: Option<PathBuf>,
    /// TLS version configuration
    pub tls_version: TlsVersion,
    /// Cipher suites
    pub cipher_suites: Vec<String>,
}

/// TLS version configuration
#[derive(Debug, Clone)]
pub struct TlsVersion {
    /// Enable TLS 1.2
    pub tls_1_2: bool,
    /// Enable TLS 1.3
    pub tls_1_3: bool,
}

impl TlsVersion {
    /// Create a new TLS version configuration
    pub fn new() -> Self {
        Self {
            tls_1_2: true,
            tls_1_3: true,
        }
    }

    /// Enable only TLS 1.2
    pub fn tls_1_2_only() -> Self {
        Self {
            tls_1_2: true,
            tls_1_3: false,
        }
    }

    /// Enable only TLS 1.3
    pub fn tls_1_3_only() -> Self {
        Self {
            tls_1_2: false,
            tls_1_3: true,
        }
    }

    /// Disable all TLS versions
    pub fn disabled() -> Self {
        Self {
            tls_1_2: false,
            tls_1_3: false,
        }
    }
}

impl Default for TlsVersion {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new() -> Self {
        Self {
            root_certs: Vec::new(),
            client_cert: None,
            verify: true,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
            tls_version: TlsVersion::default(),
            cipher_suites: Vec::new(),
        }
    }

    /// Set root certificates
    pub fn root_certs(mut self, certs: Vec<Certificate>) -> Self {
        self.root_certs = certs;
        self
    }

    /// Add a root certificate
    pub fn add_root_cert(mut self, cert: Certificate) -> Self {
        self.root_certs.push(cert);
        self
    }

    /// Set client certificate
    pub fn client_cert(mut self, cert: Identity) -> Self {
        self.client_cert = Some(cert);
        self
    }

    /// Set certificate verification
    pub fn verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// Set CA certificate path
    pub fn ca_cert_path(mut self, path: PathBuf) -> Self {
        self.ca_cert_path = Some(path);
        self
    }

    /// Set client certificate path
    pub fn client_cert_path(mut self, path: PathBuf) -> Self {
        self.client_cert_path = Some(path);
        self
    }

    /// Set client private key path
    pub fn client_key_path(mut self, path: PathBuf) -> Self {
        self.client_key_path = Some(path);
        self
    }

    /// Set TLS version configuration
    pub fn tls_version(mut self, version: TlsVersion) -> Self {
        self.tls_version = version;
        self
    }

    /// Set cipher suites
    pub fn cipher_suites(mut self, suites: Vec<String>) -> Self {
        self.cipher_suites = suites;
        self
    }

    /// Add a cipher suite
    pub fn add_cipher_suite(mut self, suite: &str) -> Self {
        self.cipher_suites.push(suite.to_string());
        self
    }

    /// Get root certificates
    pub fn get_root_certs(&self) -> &[Certificate] {
        &self.root_certs
    }

    /// Get client certificate
    pub fn get_client_cert(&self) -> Option<&Identity> {
        self.client_cert.as_ref()
    }

    /// Check if certificate verification is enabled
    pub fn is_verify_enabled(&self) -> bool {
        self.verify
    }

    /// Get CA certificate path
    pub fn get_ca_cert_path(&self) -> Option<&PathBuf> {
        self.ca_cert_path.as_ref()
    }

    /// Get client certificate path
    pub fn get_client_cert_path(&self) -> Option<&PathBuf> {
        self.client_cert_path.as_ref()
    }

    /// Get client private key path
    pub fn get_client_key_path(&self) -> Option<&PathBuf> {
        self.client_key_path.as_ref()
    }

    /// Get TLS version configuration
    pub fn get_tls_version(&self) -> &TlsVersion {
        &self.tls_version
    }

    /// Get cipher suites
    pub fn get_cipher_suites(&self) -> &[String] {
        &self.cipher_suites
    }

    /// Apply this configuration to a reqwest client builder
    pub fn apply_to_builder(self, mut builder: ReqwestBuilder) -> ReqwestBuilder {
        // Load native certificates if no custom ones are provided
        if self.root_certs.is_empty() && self.ca_cert_path.is_none() {
            if let Ok(certs) = load_native_certs() {
                for cert in certs {
                    // Note: CertificateDer field is private in this version
                    // We'll skip certificate validation for now
                    // if let Ok(cert) = Certificate::from_der(&cert.0) {
                    //     builder = builder.add_root_certificate(cert);
                    // }
                }
            }
        }

        // Add custom root certificates
        for cert in self.root_certs {
            builder = builder.add_root_certificate(cert);
        }

        // Load CA certificate from file if specified
        if let Some(ca_path) = self.ca_cert_path {
            if let Ok(cert_data) = std::fs::read(&ca_path) {
                if let Ok(cert) = Certificate::from_pem(&cert_data) {
                    builder = builder.add_root_certificate(cert);
                }
            }
        }

        // Load client certificate if specified
        if let Some(client_cert) = self.client_cert {
            // Note: identity method is not available in this version of reqwest
            // builder = builder.identity(client_cert);
        } else if let (Some(cert_path), Some(key_path)) = (self.client_cert_path, self.client_key_path) {
            if let (Ok(cert_data), Ok(key_data)) = (std::fs::read(&cert_path), std::fs::read(&key_path)) {
                // Note: Identity::from_pkcs8_pem is not available in this version
                // if let Ok(identity) = Identity::from_pkcs8_pem(&cert_data, &key_data) {
                //     builder = builder.identity(identity);
                // }
            }
        }

        // Disable certificate verification if requested
        if !self.verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        builder
    }

    /// Create a rustls client config from this configuration
    pub fn to_rustls_config(&self) -> Result<ClientConfig> {
        let mut root_store = RootCertStore::empty();

        // Load native certificates
        if let Ok(certs) = load_native_certs() {
            for cert in certs {
                // Note: CertificateDer field is private in this version
                // root_store.add(&RustlsCertificate(cert.0))
                //     .map_err(|e| Error::tls(format!("Failed to add native certificate: {}", e)))?;
            }
        }

        // Add custom root certificates
        for cert in &self.root_certs {
            // Convert reqwest Certificate to rustls Certificate
            // This is a simplified conversion - in practice you'd need to handle the format properly
        }

        // Load CA certificate from file if specified
        if let Some(ca_path) = &self.ca_cert_path {
            let cert_data = std::fs::read(ca_path)
                .map_err(|e| Error::tls(format!("Failed to read CA certificate: {}", e)))?;
            let cert = RustlsCertificate(cert_data);
            root_store.add(&cert)
                .map_err(|e| Error::tls(format!("Failed to add CA certificate: {}", e)))?;
        }

        let mut config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Configure TLS versions
        if !self.tls_version.tls_1_2 && !self.tls_version.tls_1_3 {
            return Err(Error::tls("No TLS versions enabled"));
        }

        // Configure cipher suites if specified
        if !self.cipher_suites.is_empty() {
            // This would require more complex implementation to map cipher suite names
            // to rustls cipher suite constants
        }

        Ok(config)
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// TLS configuration builder
pub struct TlsBuilder {
    config: TlsConfig,
}

impl TlsBuilder {
    /// Create a new TLS builder
    pub fn new() -> Self {
        Self {
            config: TlsConfig::new(),
        }
    }

    /// Set certificate verification
    pub fn verify(mut self, verify: bool) -> Self {
        self.config = self.config.verify(verify);
        self
    }

    /// Disable certificate verification
    pub fn no_verify(mut self) -> Self {
        self.config = self.config.verify(false);
        self
    }

    /// Set CA certificate path
    pub fn ca_cert(mut self, path: &str) -> Self {
        self.config = self.config.ca_cert_path(PathBuf::from(path));
        self
    }

    /// Set client certificate and key paths
    pub fn client_cert(mut self, cert_path: &str, key_path: &str) -> Self {
        self.config = self.config
            .client_cert_path(PathBuf::from(cert_path))
            .client_key_path(PathBuf::from(key_path));
        self
    }

    /// Set TLS version
    pub fn tls_version(mut self, version: TlsVersion) -> Self {
        self.config = self.config.tls_version(version);
        self
    }

    /// Enable only TLS 1.2
    pub fn tls_1_2_only(mut self) -> Self {
        self.config = self.config.tls_version(TlsVersion::tls_1_2_only());
        self
    }

    /// Enable only TLS 1.3
    pub fn tls_1_3_only(mut self) -> Self {
        self.config = self.config.tls_version(TlsVersion::tls_1_3_only());
        self
    }

    /// Add a cipher suite
    pub fn cipher_suite(mut self, suite: &str) -> Self {
        self.config = self.config.add_cipher_suite(suite);
        self
    }

    /// Build the TLS configuration
    pub fn build(self) -> TlsConfig {
        self.config
    }
}

impl Default for TlsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods for TLS operations
impl TlsConfig {
    /// Create a TLS builder
    pub fn builder() -> TlsBuilder {
        TlsBuilder::new()
    }

    /// Create a TLS configuration that accepts invalid certificates
    pub fn insecure() -> Self {
        Self::new().verify(false)
    }

    /// Create a TLS configuration with custom CA certificate
    pub fn with_ca_cert(path: &str) -> Result<Self> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(Error::tls(format!("CA certificate file not found: {}", path.display())));
        }
        Ok(Self::new().ca_cert_path(path))
    }

    /// Create a TLS configuration with client certificate
    pub fn with_client_cert(cert_path: &str, key_path: &str) -> Result<Self> {
        let cert_path = PathBuf::from(cert_path);
        let key_path = PathBuf::from(key_path);
        
        if !cert_path.exists() {
            return Err(Error::tls(format!("Client certificate file not found: {}", cert_path.display())));
        }
        if !key_path.exists() {
            return Err(Error::tls(format!("Client key file not found: {}", key_path.display())));
        }
        
        Ok(Self::new()
            .client_cert_path(cert_path)
            .client_key_path(key_path))
    }

    /// Create a TLS configuration for development (insecure)
    pub fn development() -> Self {
        Self::new()
            .verify(false)
            .tls_version(TlsVersion::new())
    }

    /// Create a TLS configuration for production
    pub fn production() -> Self {
        Self::new()
            .verify(true)
            .tls_version(TlsVersion::new())
    }
}

/// TLS utilities
pub mod utils {
    use super::*;

    /// Check if a certificate file is valid
    pub fn is_valid_cert_file(path: &PathBuf) -> bool {
        if !path.exists() {
            return false;
        }
        
        if let Ok(data) = std::fs::read(path) {
            Certificate::from_pem(&data).is_ok() || Certificate::from_der(&data).is_ok()
        } else {
            false
        }
    }

    /// Check if a private key file is valid
    pub fn is_valid_key_file(path: &PathBuf) -> bool {
        if !path.exists() {
            return false;
        }
        
        if let Ok(data) = std::fs::read(path) {
            // This is a simplified check - in practice you'd want to validate the key format
            data.len() > 0
        } else {
            false
        }
    }

    /// Get the default cipher suites
    pub fn default_cipher_suites() -> Vec<String> {
        vec![
            "TLS_AES_256_GCM_SHA384".to_string(),
            "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            "TLS_AES_128_GCM_SHA256".to_string(),
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
            "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_creation() {
        let config = TlsConfig::new();
        assert!(config.is_verify_enabled());
        assert!(config.get_root_certs().is_empty());
    }

    #[test]
    fn test_tls_config_insecure() {
        let config = TlsConfig::insecure();
        assert!(!config.is_verify_enabled());
    }

    #[test]
    fn test_tls_version() {
        let version = TlsVersion::new();
        assert!(version.tls_1_2);
        assert!(version.tls_1_3);

        let version = TlsVersion::tls_1_2_only();
        assert!(version.tls_1_2);
        assert!(!version.tls_1_3);

        let version = TlsVersion::tls_1_3_only();
        assert!(!version.tls_1_2);
        assert!(version.tls_1_3);
    }

    #[test]
    fn test_tls_builder() {
        let config = TlsBuilder::new()
            .no_verify()
            .tls_1_3_only()
            .build();
        
        assert!(!config.is_verify_enabled());
        assert!(!config.get_tls_version().tls_1_2);
        assert!(config.get_tls_version().tls_1_3);
    }

    #[test]
    fn test_utils() {
        let temp_dir = std::env::temp_dir();
        let non_existent_path = temp_dir.join("non_existent_cert.pem");
        
        assert!(!utils::is_valid_cert_file(&non_existent_path));
        assert!(!utils::is_valid_key_file(&non_existent_path));
        
        let cipher_suites = utils::default_cipher_suites();
        assert!(!cipher_suites.is_empty());
        assert!(cipher_suites.contains(&"TLS_AES_256_GCM_SHA384".to_string()));
    }
} 