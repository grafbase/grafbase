use std::time::Duration;

/// Defines if and how some data should be cached
/// Fields should match the ones in the Cache-Control HTTP header if their name are identical.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct OperationCacheControl {
    pub max_age: Duration,
    pub max_stale: Duration,
    // sorted to ensure consistent cache key
    scopes: Vec<CacheScopeDefinition>,
}

impl OperationCacheControl {
    pub fn scopes(&self) -> &Vec<CacheScopeDefinition> {
        &self.scopes
    }

    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    pub fn with_max_stale(mut self, max_stale: Duration) -> Self {
        self.max_stale = max_stale;
        self
    }

    pub fn with_scopes(mut self, mut scopes: Vec<CacheScopeDefinition>) -> Self {
        scopes.sort_unstable(); // We have total ordering, so unstable is good
        self.scopes = scopes;
        self
    }

    pub fn is_private(&self) -> bool {
        self.scopes.is_empty()
    }

    pub fn to_response_header(&self) -> headers::CacheControl {
        let mut header = headers::CacheControl::new();
        if !self.max_age.is_zero() {
            header = header.with_max_age(self.max_age)
        }
        if !self.max_stale.is_zero() {
            header = header.with_max_stale(self.max_stale);
        }
        if self.is_private() {
            header = header.with_private();
        } else {
            header = header.with_public();
        }
        header
    }
}

/// Cache scope define for who the cache should be accessible.
/// If two request have identical scopes values (same JWT claim for example), their cached response
/// are shared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheScopeDefinition {
    Public,
    Authenticated,
    JwtClaim { path: Vec<String> },
    HeaderValue { name: http::HeaderName },
}

impl CacheScopeDefinition {
    /// Stable order of the cache scopes. It must be consistent over time to ensure we do generate
    /// the same cache key
    pub fn stable_id(&self) -> u8 {
        match self {
            CacheScopeDefinition::Public => 0,
            CacheScopeDefinition::JwtClaim { .. } => 1,
            CacheScopeDefinition::HeaderValue { .. } => 2,
            CacheScopeDefinition::Authenticated => 3,
        }
    }
}

impl PartialOrd for CacheScopeDefinition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CacheScopeDefinition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (CacheScopeDefinition::JwtClaim { path: left }, CacheScopeDefinition::JwtClaim { path: right }) => {
                left.cmp(right)
            }
            (CacheScopeDefinition::HeaderValue { name: left }, CacheScopeDefinition::HeaderValue { name: right }) => {
                left.as_str().cmp(right.as_str())
            }
            (left, right) => left.stable_id().cmp(&right.stable_id()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_control_scopes_ordering() {
        let mut control = OperationCacheControl::default();
        control = control.with_scopes(vec![
            CacheScopeDefinition::HeaderValue {
                name: http::header::ACCEPT,
            },
            CacheScopeDefinition::Public,
            CacheScopeDefinition::JwtClaim {
                path: vec!["b".to_string()],
            },
            CacheScopeDefinition::HeaderValue {
                name: http::header::AUTHORIZATION,
            },
            CacheScopeDefinition::JwtClaim {
                path: vec!["a".to_string()],
            },
        ]);
        assert_eq!(
            control.scopes(),
            &vec![
                CacheScopeDefinition::Public,
                CacheScopeDefinition::JwtClaim {
                    path: vec!["a".to_string()],
                },
                CacheScopeDefinition::JwtClaim {
                    path: vec!["b".to_string()],
                },
                CacheScopeDefinition::HeaderValue {
                    name: http::header::ACCEPT,
                },
                CacheScopeDefinition::HeaderValue {
                    name: http::header::AUTHORIZATION,
                },
            ]
        );
    }
}
