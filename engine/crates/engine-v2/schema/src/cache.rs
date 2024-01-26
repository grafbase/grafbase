use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct CacheConfig {
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
}

pub trait Merge<T> {
    fn merge(self, element: T) -> T;
}

impl Merge<CacheConfig> for CacheConfig {
    fn merge(self, right: CacheConfig) -> CacheConfig {
        CacheConfig {
            max_age: self.max_age.min(right.max_age),
            stale_while_revalidate: self.stale_while_revalidate.min(right.stale_while_revalidate),
        }
    }
}

impl Merge<Option<CacheConfig>> for Option<CacheConfig> {
    fn merge(self, right: Option<CacheConfig>) -> Option<CacheConfig> {
        self.and_then(|left| right.map(|right| left.merge(right)).or(Some(left)))
            .or(right)
    }
}

impl From<config::latest::CacheConfig> for CacheConfig {
    fn from(value: config::latest::CacheConfig) -> Self {
        CacheConfig {
            max_age: value.max_age,
            stale_while_revalidate: value.stale_while_revalidate,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{CacheConfig, Merge};
    use std::time::Duration;

    #[test]
    fn test_merge() {
        let left = CacheConfig {
            max_age: Duration::from_secs(1),
            stale_while_revalidate: Duration::from_secs(1),
        };

        let right = CacheConfig {
            max_age: Duration::from_secs(2),
            stale_while_revalidate: Duration::from_secs(2),
        };

        assert_eq!(left, left.merge(right));
    }

    #[test]
    fn test_merge_optional() {
        let left = Some(CacheConfig {
            max_age: Duration::from_secs(1),
            stale_while_revalidate: Duration::from_secs(1),
        });

        let right = Some(CacheConfig {
            max_age: Duration::from_secs(2),
            stale_while_revalidate: Duration::from_secs(2),
        });

        assert_eq!(left, left.merge(right));
        assert_eq!(left, left.merge(None));
        assert_eq!(right, None.merge(right));
    }
}
