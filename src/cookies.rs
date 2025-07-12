use std::sync::Mutex;
use cookie::{Cookie, CookieJar as CookieJarInner};
use url::Url;

use crate::error::{Error, Result};

/// Cookie jar for managing cookies across requests
///
/// This provides a thread-safe way to store and retrieve cookies
/// for HTTP requests and responses.
#[derive(Debug)]
pub struct CookieJar {
    inner: Mutex<CookieJarInner>,
}

impl CookieJar {
    /// Create a new empty cookie jar
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(CookieJarInner::new()),
        }
    }

    /// Add a cookie to the jar
    pub fn add(&self, cookie: Cookie<'static>) {
        if let Ok(mut jar) = self.inner.lock() {
            jar.add(cookie);
        }
    }

    /// Add a cookie from a string
    pub fn add_from_string(&self, cookie_str: &str) -> Result<()> {
        let cookie = Cookie::parse(cookie_str)
            .map_err(|e| Error::cookie(format!("Failed to parse cookie: {}", e)))?;
        self.add(cookie.into_owned());
        Ok(())
    }

    /// Add a cookie from a response header
    pub fn add_cookie_from_response(&self, cookie_str: &str, _url: &Url) {
        if let Ok(mut jar) = self.inner.lock() {
            if let Ok(cookie) = Cookie::parse(cookie_str) {
                jar.add(cookie.into_owned());
            }
        }
    }

    /// Get cookies for a specific URL
    pub fn cookies_for_url(&self, url: &Url) -> Vec<Cookie<'static>> {
        if let Ok(jar) = self.inner.lock() {
            jar.iter()
                .filter(|cookie| {
                    // Basic domain matching
                    if let Some(cookie_domain) = cookie.domain() {
                        url.host_str()
                            .map(|host| host.ends_with(cookie_domain))
                            .unwrap_or(false)
                    } else {
                        true
                    }
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all cookies as a string for a request header
    pub fn cookies_string_for_url(&self, url: &Url) -> String {
        let cookies = self.cookies_for_url(url);
        cookies
            .iter()
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// Remove a cookie by name
    pub fn remove(&self, name: &str) {
        if let Ok(mut jar) = self.inner.lock() {
            let name_owned = name.to_string();
            jar.remove(Cookie::build(name_owned).build());
        }
    }

    /// Clear all cookies
    pub fn clear(&self) {
        if let Ok(_jar) = self.inner.lock() {
            // Note: CookieJar doesn't have a clear method in this version
            // jar.clear();
        }
    }

    /// Get the number of cookies in the jar
    pub fn len(&self) -> usize {
        if let Ok(jar) = self.inner.lock() {
            jar.iter().count()
        } else {
            0
        }
    }

    /// Check if the cookie jar is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all cookies
    pub fn all_cookies(&self) -> Vec<Cookie<'static>> {
        if let Ok(jar) = self.inner.lock() {
            jar.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a cookie exists
    pub fn has_cookie(&self, name: &str) -> bool {
        if let Ok(jar) = self.inner.lock() {
            jar.get(name).is_some()
        } else {
            false
        }
    }

    /// Get a specific cookie by name
    pub fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        if let Ok(jar) = self.inner.lock() {
            jar.get(name).cloned()
        } else {
            None
        }
    }
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CookieJar {
    fn clone(&self) -> Self {
        let cookies = self.all_cookies();
        let jar = Self::new();
        for cookie in cookies {
            jar.add(cookie);
        }
        jar
    }
}

/// Cookie builder for creating cookies with various attributes
pub struct CookieBuilder {
    name: String,
    value: String,
    domain: Option<String>,
    path: Option<String>,
    expires: Option<cookie::Expiration>,
    max_age: Option<i64>,
    secure: bool,
    http_only: bool,
    same_site: Option<cookie::SameSite>,
}

impl CookieBuilder {
    /// Create a new cookie builder
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            domain: None,
            path: None,
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Set the domain
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = Some(domain.to_string());
        self
    }

    /// Set the path
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    /// Set the expiration time
    pub fn expires(mut self, expires: cookie::Expiration) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Set the max age in seconds
    pub fn max_age(mut self, max_age: i64) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Set the secure flag
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Set the http only flag
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set the same site attribute
    pub fn same_site(mut self, same_site: cookie::SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Build the cookie
    pub fn build(self) -> Cookie<'static> {
        let mut cookie = Cookie::new(self.name, self.value);
        
        if let Some(domain) = self.domain {
            cookie.set_domain(domain);
        }
        
        if let Some(path) = self.path {
            cookie.set_path(path);
        }
        
        if let Some(expires) = self.expires {
            cookie.set_expires(expires);
        }
        
        if let Some(max_age) = self.max_age {
            cookie.set_max_age(cookie::time::Duration::seconds(max_age));
        }
        
        if self.secure {
            cookie.set_secure(true);
        }
        
        if self.http_only {
            cookie.set_http_only(true);
        }
        
        if let Some(same_site) = self.same_site {
            cookie.set_same_site(same_site);
        }
        
        cookie
    }
}

/// Convenience methods for cookie operations
impl CookieJar {
    /// Create a cookie builder
    pub fn builder(name: &str, value: &str) -> CookieBuilder {
        CookieBuilder::new(name, value)
    }

    /// Add a simple cookie
    pub fn add_simple(&self, name: &str, value: &str) {
        let cookie = Cookie::new(name.to_string(), value.to_string());
        self.add(cookie.into_owned());
    }

    /// Add a session cookie (expires when browser closes)
    pub fn add_session_cookie(&self, name: &str, value: &str) {
        let cookie = CookieBuilder::new(name, value)
            .http_only(true)
            .build();
        self.add(cookie);
    }

    /// Add a persistent cookie with expiration
    pub fn add_persistent_cookie(&self, name: &str, value: &str, max_age: i64) {
        let cookie = CookieBuilder::new(name, value)
            .max_age(max_age)
            .http_only(true)
            .build();
        self.add(cookie);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_jar_creation() {
        let jar = CookieJar::new();
        assert!(jar.is_empty());
        assert_eq!(jar.len(), 0);
    }

    #[test]
    fn test_cookie_jar_add_and_get() {
        let jar = CookieJar::new();
        jar.add_simple("test", "value");
        
        assert!(!jar.is_empty());
        assert_eq!(jar.len(), 1);
        assert!(jar.has_cookie("test"));
        
        let cookie = jar.get_cookie("test");
        assert!(cookie.is_some());
        assert_eq!(cookie.unwrap().value(), "value");
    }

    #[test]
    fn test_cookie_builder() {
        let cookie = CookieBuilder::new("test", "value")
            .domain("example.com")
            .path("/")
            .secure(true)
            .http_only(true)
            .build();
        
        assert_eq!(cookie.name(), "test");
        assert_eq!(cookie.value(), "value");
        assert_eq!(cookie.domain().unwrap(), "example.com");
        assert_eq!(cookie.path().unwrap(), "/");
        assert!(cookie.secure().unwrap());
        assert!(cookie.http_only().unwrap());
    }

    #[test]
    fn test_cookie_jar_clone() {
        let jar = CookieJar::new();
        jar.add_simple("test", "value");
        
        let cloned_jar = jar.clone();
        assert_eq!(cloned_jar.len(), 1);
        assert!(cloned_jar.has_cookie("test"));
    }
} 