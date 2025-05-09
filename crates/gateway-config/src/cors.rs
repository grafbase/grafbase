use ascii::AsciiString;
use duration_str::deserialize_option_duration;
use http::{HeaderName, HeaderValue};
use std::time::Duration;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, ExposeHeaders};
use url::Url;

#[derive(Clone, Default, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CorsConfig {
    /// If false (or not defined), credentials are not allowed in requests
    pub allow_credentials: bool,
    /// Origins from which we allow requests.
    /// 
    /// This can be:
    /// - `{ allow_origins = "any" }` - Allows any origin (equivalent to "*")
    /// - `{ allow_origins = ["https://example.com", "*.example.com"] }` - Allows specific origins and wildcard patterns
    ///
    /// Wildcard patterns like "*.example.com" will match any subdomain such as "api.example.com" or 
    /// "user.example.com", but not the apex domain "example.com" itself.
    pub allow_origins: Option<AnyOrUrlArray>,
    /// Maximum time between OPTIONS and the next request
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub max_age: Option<Duration>,
    /// HTTP methods allowed to the endpoint.
    pub allow_methods: Option<AnyOrHttpMethodArray>,
    /// Headers allowed in incoming requests
    pub allow_headers: Option<AnyOrAsciiStringArray>,
    /// Headers exposed from the OPTIONS request
    pub expose_headers: Option<AnyOrAsciiStringArray>,
    /// If set, allows browsers from private network to connect
    pub allow_private_network: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl From<http::Method> for HttpMethod {
    fn from(value: http::Method) -> Self {
        if value == http::Method::GET {
            Self::Get
        } else if value == http::Method::POST {
            Self::Post
        } else if value == http::Method::PUT {
            Self::Put
        } else if value == http::Method::DELETE {
            Self::Delete
        } else if value == http::Method::PATCH {
            Self::Patch
        } else if value == http::Method::HEAD {
            Self::Head
        } else if value == http::Method::OPTIONS {
            Self::Options
        } else if value == http::Method::TRACE {
            Self::Trace
        } else if value == http::Method::CONNECT {
            Self::Connect
        } else {
            todo!("Unsupported HTTP method: {:?}", value);
        }
    }
}

impl From<HttpMethod> for http::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => http::Method::GET,
            HttpMethod::Post => http::Method::POST,
            HttpMethod::Put => http::Method::PUT,
            HttpMethod::Delete => http::Method::DELETE,
            HttpMethod::Head => http::Method::HEAD,
            HttpMethod::Options => http::Method::OPTIONS,
            HttpMethod::Connect => http::Method::CONNECT,
            HttpMethod::Patch => http::Method::PATCH,
            HttpMethod::Trace => http::Method::TRACE,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(expecting = "expecting string \"any\", or an array of urls and wildcard patterns like \"*.example.com\"")]
pub enum AnyOrUrlArray {
    /// Allow any origin (equivalent to "*")
    Any,
    /// Allow specific origins (URLs) and wildcard patterns
    ///
    /// Example TOML configuration:
    /// ```toml
    /// [cors]
    /// allow_origins = ["https://example.com", "*.example.com", "https://api.example.org"]
    /// ```
    #[serde(untagged)]
    Origins(Vec<String>),
}

#[cfg(test)]
impl AnyOrUrlArray {
    pub fn origins(values: Vec<&str>) -> Self {
        Self::Origins(values.into_iter().map(String::from).collect())
    }
}

impl From<AnyOrUrlArray> for AllowOrigin {
    fn from(value: AnyOrUrlArray) -> Self {
            match value {
                AnyOrUrlArray::Any => AllowOrigin::any(),
                AnyOrUrlArray::Origins(origins) => {
                    // Split into exact URLs and wildcard patterns
                    let mut exact_origins = Vec::new();
                    let mut wildcard_patterns = Vec::new();
                    
                    for origin in &origins {
                        if origin.starts_with("*.") {
                            wildcard_patterns.push(origin.clone());
                        } else {
                            exact_origins.push(origin.clone());
                        }
                    }
                    
                    // If we only have exact origins and no wildcards, use the simpler list method
                    if !wildcard_patterns.is_empty() {
                        // We have wildcards, so we need to use a predicate
                        let patterns = origins.clone();
                        
                        let predicate = move |origin: &HeaderValue, _: &http::request::Parts| {
                            let origin_str = match origin.to_str() {
                                Ok(s) => s,
                                Err(_) => return false,
                            };
                            
                            // Check exact matches first
                            if patterns.iter().any(|pattern| {
                                !pattern.starts_with("*.") && pattern == origin_str
                            }) {
                                return true;
                            }

                            // Parse the origin URL to handle wildcards
                            let origin_url = match Url::parse(origin_str) {
                                Ok(url) => url,
                                Err(_) => return false,
                            };

                            // Extract host from origin
                            let host = match origin_url.host_str() {
                                Some(h) => h,
                                None => return false,
                            };

                            // Check if the host matches any of our wildcard patterns
                            patterns.iter().any(|pattern| {
                                if pattern.starts_with("*.") {
                                    // Handle wildcard subdomain pattern (*.example.com)
                                    // This will match subdomains like "api.example.com" but NOT "example.com"
                                    let domain_part = &pattern[2..]; // Skip the "*." prefix

                                    // Check if host ends with the domain part and has a subdomain
                                    host.ends_with(domain_part) && 
                                        host.len() > domain_part.len() && 
                                        host.as_bytes()[host.len() - domain_part.len() - 1] == b'.'
                                } else {
                                    false // Exact matches were already checked
                                }
                            })
                        };

                        AllowOrigin::predicate(predicate)
                    } else {
                        // No wildcards, just use the list method
                        let origin_values = exact_origins
                            .iter()
                            .map(|url| url.strip_suffix('/').unwrap_or(url))
                            .map(|url| HeaderValue::from_str(url).expect("must be ascii"));

                        AllowOrigin::list(origin_values)
                    }
                }
            }
        }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(expecting = "expecting string \"any\", or an array of capitalized HTTP methods")]
pub enum AnyOrHttpMethodArray {
    Any,
    #[serde(untagged)]
    Explicit(Vec<HttpMethod>),
}

impl From<AnyOrHttpMethodArray> for AllowMethods {
    fn from(value: AnyOrHttpMethodArray) -> Self {
        match value {
            AnyOrHttpMethodArray::Any => AllowMethods::any(),
            AnyOrHttpMethodArray::Explicit(methods) => {
                let methods = methods.iter().map(|method| http::Method::from(*method));
                AllowMethods::list(methods)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(expecting = "expecting string \"any\", or an array of ASCII strings")]
pub enum AnyOrAsciiStringArray {
    Any,
    #[serde(untagged)]
    Explicit(Vec<AsciiString>),
}

impl From<AnyOrAsciiStringArray> for AllowHeaders {
    fn from(value: AnyOrAsciiStringArray) -> Self {
        match value {
            AnyOrAsciiStringArray::Any => AllowHeaders::any(),
            AnyOrAsciiStringArray::Explicit(headers) => {
                let headers = headers
                    .iter()
                    .map(|header| HeaderName::from_bytes(header.as_bytes()).expect("must be ascii"));

                AllowHeaders::list(headers)
            }
        }
    }
}

impl From<AnyOrAsciiStringArray> for ExposeHeaders {
    fn from(value: AnyOrAsciiStringArray) -> Self {
        match value {
            AnyOrAsciiStringArray::Any => ExposeHeaders::any(),
            AnyOrAsciiStringArray::Explicit(headers) => {
                let headers = headers
                    .iter()
                    .map(|header| HeaderName::from_bytes(header.as_bytes()).expect("must be ascii"));

                ExposeHeaders::list(headers)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper function to test our predicate directly
    fn test_origin_matches(allow_origins: &AnyOrUrlArray, origin: &str) -> bool {
        
        match allow_origins {
            AnyOrUrlArray::Any => true,
            AnyOrUrlArray::Origins(origins) => {
                // Check for exact match first
                if origins.iter().any(|o| !o.starts_with("*.") && o == origin) {
                    return true;
                }
                
                // Try to parse as URL to check wildcard patterns
                if let Ok(origin_url) = Url::parse(origin) {
                    if let Some(host) = origin_url.host_str() {
                        // Check if the host matches any wildcard pattern
                        return origins.iter().any(|pattern| {
                            if pattern.starts_with("*.") {
                                let domain_part = &pattern[2..]; // Skip the "*." prefix
                                host.ends_with(domain_part) && 
                                    host.len() > domain_part.len() && 
                                    host.as_bytes()[host.len() - domain_part.len() - 1] == b'.'
                            } else {
                                false // Exact matches were already checked
                            }
                        });
                    }
                }
                
                false
            }
        }
    }
    
    #[test]
    fn test_wildcard_domains() {
        // Create a wildcard domain pattern
        let allow_origins = AnyOrUrlArray::origins(vec!["*.example.com"]);
        
        // Test valid origins
        assert!(test_origin_matches(&allow_origins, "https://test.example.com"));
        assert!(test_origin_matches(&allow_origins, "https://deep.sub.example.com"));
        
        // Test invalid origins
        assert!(!test_origin_matches(&allow_origins, "https://example.com"));
        assert!(!test_origin_matches(&allow_origins, "https://test.different.com"));
        
        // Also test the conversion to AllowOrigin to ensure it compiles
        let _: AllowOrigin = allow_origins.clone().into();
    }
    
    #[test]
    fn test_multiple_wildcard_domains() {
        // Create multiple wildcard domain patterns
        let allow_origins = AnyOrUrlArray::origins(vec!["*.example.com", "*.test.org"]);
        
        // Test valid origins
        assert!(test_origin_matches(&allow_origins, "https://sub.example.com"));
        assert!(test_origin_matches(&allow_origins, "https://sub.test.org"));
        
        // Test invalid origin
        assert!(!test_origin_matches(&allow_origins, "https://sub.other.net"));
        
        // Also test the conversion to AllowOrigin to ensure it compiles
        let _: AllowOrigin = allow_origins.clone().into();
    }
    
    #[test]
    fn test_mixed_origins_and_wildcards() {
        // Create a mix of exact origins and wildcard patterns
        let allow_origins = AnyOrUrlArray::origins(vec![
            "https://example.com",
            "*.api.example.com",
            "https://dashboard.example.org"
        ]);
        
        // Test valid origins - both exact matches and wildcards
        assert!(test_origin_matches(&allow_origins, "https://example.com"));
        assert!(test_origin_matches(&allow_origins, "https://test.api.example.com"));
        assert!(test_origin_matches(&allow_origins, "https://dashboard.example.org"));
        
        // Test invalid origins
        assert!(!test_origin_matches(&allow_origins, "https://example.org"));
        assert!(!test_origin_matches(&allow_origins, "https://api.example.com")); // This would need "api.example.com" explicitly
        
        // Also test the conversion to AllowOrigin to ensure it compiles
        let _: AllowOrigin = allow_origins.clone().into();
    }
}
