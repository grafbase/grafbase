use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CacheControl {
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
}

impl CacheControl {
    pub fn union(self, other: CacheControl) -> CacheControl {
        CacheControl {
            max_age: self.max_age.min(other.max_age),
            stale_while_revalidate: self.stale_while_revalidate.min(other.stale_while_revalidate),
        }
    }

    pub fn union_opt(left: Option<&CacheControl>, right: Option<&CacheControl>) -> Option<CacheControl> {
        match (left, right) {
            (Some(left), Some(right)) => Some(left.union(*right)),
            (Some(left), None) => Some(*left),
            (None, Some(right)) => Some(*right),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge() {
        let left = CacheControl {
            max_age: Duration::from_secs(1),
            stale_while_revalidate: Duration::from_secs(1),
        };

        let right = CacheControl {
            max_age: Duration::from_secs(2),
            stale_while_revalidate: Duration::from_secs(2),
        };

        assert_eq!(left, left.union(right));
    }

    #[test]
    fn test_merge_optional() {
        let left = Some(CacheControl {
            max_age: Duration::from_secs(1),
            stale_while_revalidate: Duration::from_secs(1),
        });

        let right = Some(CacheControl {
            max_age: Duration::from_secs(2),
            stale_while_revalidate: Duration::from_secs(2),
        });

        assert_eq!(left, CacheControl::union_opt(left.as_ref(), right.as_ref()));
        assert_eq!(left, CacheControl::union_opt(left.as_ref(), None));
        assert_eq!(right, CacheControl::union_opt(None, right.as_ref()));
    }
}
