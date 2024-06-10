use std::collections::BTreeSet;

#[serde_with::minify_field_names(serialize = "minified", deserialize = "minified")]
#[serde_with::skip_serializing_defaults(Option, bool, usize)]
#[derive(Default, Hash, Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize)]
pub struct CacheControl {
    /// Scope is public, default is false.
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,

    /// Cache stale_while_revalidate, default is 0.
    pub stale_while_revalidate: usize,

    /// Invalidation policy for mutations, default is None.
    pub invalidation_policy: Option<CacheInvalidationPolicy>,

    /// Access scopes
    pub access_scopes: Option<BTreeSet<CacheAccessScope>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum CacheAccessScope {
    ApiKey,
    Jwt { claim: String },
    Header { header: String },
    Public,
}

impl CacheControl {
    pub fn merge(&mut self, mut other: CacheControl) {
        *self = CacheControl {
            public: self.public && other.public,
            max_age: self.max_age.min(other.max_age),
            stale_while_revalidate: self.stale_while_revalidate.min(other.stale_while_revalidate),
            invalidation_policy: if self.invalidation_policy.is_none() {
                other.invalidation_policy
            } else if other.invalidation_policy.is_none() {
                self.invalidation_policy.take()
            } else {
                let self_policy = self.invalidation_policy.take().unwrap();
                let other_policy = other.invalidation_policy.take().unwrap();
                Some(self_policy.max(other_policy))
            },
            access_scopes: if self.access_scopes.is_none() {
                other.access_scopes
            } else if other.access_scopes.is_none() {
                self.access_scopes.take()
            } else {
                let mut self_scopes = self.access_scopes.take().unwrap();
                let other_scopes = other.access_scopes.unwrap();
                self_scopes.extend(other_scopes);
                Some(self_scopes)
            },
        };
    }
}
