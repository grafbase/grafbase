#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub enum ComplexityControl {
    #[default]
    Disabled,
    /// Complexity limits are enforced with the given limit
    Enforce { limit: usize, list_size: usize },
    /// Complexity limits are measured and reported.
    ///
    /// A limit can still be provided which we should use for
    /// reporting whether something would have gone over the limit.
    Measure { limit: Option<usize>, list_size: usize },
}
