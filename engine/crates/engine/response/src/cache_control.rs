#[derive(Default, Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize)]
pub struct CacheControl {
    /// Scope is public, default is false.
    #[serde(default)]
    pub public: bool,

    /// Cache max age, default is 0.
    #[serde(default)]
    pub max_age: usize,

    /// Cache stale_while_revalidate, default is 0.
    #[serde(default)]
    pub stale_while_revalidate: usize,
}

impl CacheControl {
    pub fn merge(&mut self, other: CacheControl) {
        *self = CacheControl {
            public: self.public && other.public,
            max_age: self.max_age.min(other.max_age),
            stale_while_revalidate: self.stale_while_revalidate.min(other.stale_while_revalidate),
        };
    }
}
