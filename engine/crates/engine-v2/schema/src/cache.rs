#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct CacheConfig {
    pub max_age: usize,
    pub stale_while_revalidate: usize,
}

pub trait Merge<T> {
    fn merge(self, element: T) -> T;
}

impl Merge<CacheConfig> for CacheConfig {
    fn merge(self, right: CacheConfig) -> CacheConfig {
        CacheConfig {
            max_age: if self.max_age == 0 {
                right.max_age
            } else if right.max_age == 0 {
                self.max_age
            } else {
                self.max_age.min(right.max_age)
            },
            stale_while_revalidate: if self.stale_while_revalidate == 0 {
                right.stale_while_revalidate
            } else if right.stale_while_revalidate == 0 {
                self.stale_while_revalidate
            } else {
                self.stale_while_revalidate.min(right.stale_while_revalidate)
            },
        }
    }
}

impl Merge<Option<CacheConfig>> for Option<CacheConfig> {
    fn merge(self, right: Option<CacheConfig>) -> Option<CacheConfig> {
        self
            .and_then(|left| right
                .map(|right| left.merge(right))
                .or(Some(left))
            )
            .or(right)
    }
}

impl From<&config::latest::CacheConfig> for CacheConfig {
    fn from(value: &config::latest::CacheConfig) -> Self {
        CacheConfig {
            max_age: value.max_age,
            stale_while_revalidate: value.stale_while_revalidate,
        }
    }
}
