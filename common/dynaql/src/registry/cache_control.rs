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
#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, bool, usize)]
#[derive(Default, Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize, Hash)]
pub struct CacheControl {
    /// Scope is public, default is false.
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,

    /// Cache stale_while_revalidate, default is 0.
    pub stale_while_revalidate: usize,

    /// Invalidation policy for mutations, default is None.
    pub invalidation_policy: Option<CacheInvalidationPolicy>,
}

impl CacheControl {
    pub(crate) fn merge(&mut self, other: CacheControl) {
        *self = CacheControl {
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
            invalidation_policy: if self.invalidation_policy.is_none() {
                other.invalidation_policy
            } else if other.invalidation_policy.is_none() {
                self.invalidation_policy.clone()
            } else {
                let self_policy = self.invalidation_policy.as_ref().cloned().unwrap();
                let other_policy = other.invalidation_policy.as_ref().cloned().unwrap();
                Some(self_policy.max(other_policy))
            },
        };
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
/// Represents cache purge behaviour for mutations
/// The order of variants is significant, from highest to lowest specificity
pub enum CacheInvalidationPolicy {
    /// Mutations for the target type will invalidate all cache values that have the chosen identifier
    /// E.g:
    /// with a mutation policy { policy: Entity, field: id }
    /// a mutation for a Post returns a Post { id: "1234" }, all cache values that have a Post#id:1234 will be invalidated
    Entity { field: String },
    /// Mutations for the target type will invalidate all cache values that have lists of the type in them
    /// Post#List
    List,
    /// Mutations for the target type will invalidate all cache values that have Type in them
    /// Post
    Type,
}
