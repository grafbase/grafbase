/// Cache control values
///
/// # Examples
///
/// ```rust, ignore
/// use dynaql::*;
///
/// struct Query;
///
/// #[Object(cache_control(max_age = 60))]
/// impl Query {
///     #[graphql(cache_control(max_age = 30))]
///     async fn value1(&self) -> i32 {
///         0
///     }
///
///     #[graphql(cache_control(private))]
///     async fn value2(&self) -> i32 {
///         0
///     }
/// }
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
/// assert_eq!(schema.execute("{ value1 }").await.into_result().unwrap().cache_control, CacheControl { public: true, max_age: 30 });
/// assert_eq!(schema.execute("{ value2 }").await.into_result().unwrap().cache_control, CacheControl { public: false, max_age: 60 });
/// assert_eq!(schema.execute("{ value1 value2 }").await.into_result().unwrap().cache_control, CacheControl { public: false, max_age: 30 });
/// # });
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub struct CacheControl {
    /// Scope is public, default is true.
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,

    /// Cache stale_while_revalidate, default is 0.
    #[serde(default)]
    pub stale_while_revalidate: usize,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            public: true,
            max_age: 0,
            stale_while_revalidate: 0,
        }
    }
}

impl CacheControl {
    #[must_use]
    pub(crate) fn merge(self, other: &CacheControl) -> CacheControl {
        CacheControl {
            public: self.public && other.public,
            max_age: if self.max_age == 0 {
                other.max_age
            } else if other.max_age == 0 {
                self.max_age
            } else {
                self.max_age.min(other.max_age)
            },
            stale_while_revalidate: if self.stale_while_revalidate == 0 {
                other.stale_while_revalidate
            } else if other.stale_while_revalidate == 0 {
                self.stale_while_revalidate
            } else {
                self.stale_while_revalidate
                    .min(other.stale_while_revalidate)
            },
        }
    }
}
