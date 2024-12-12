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

impl ComplexityControl {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, ComplexityControl::Disabled)
    }

    pub fn is_disabled(&self) -> bool {
        matches!(self, ComplexityControl::Disabled)
    }

    pub fn is_enforce(&self) -> bool {
        matches!(self, ComplexityControl::Enforce { .. })
    }

    pub fn limit(&self) -> Option<usize> {
        match self {
            ComplexityControl::Disabled => None,
            ComplexityControl::Enforce { limit, .. } => Some(*limit),
            ComplexityControl::Measure { limit, .. } => *limit,
        }
    }

    pub fn list_size(&self) -> Option<usize> {
        match self {
            ComplexityControl::Disabled => None,
            ComplexityControl::Enforce { list_size, .. } => Some(*list_size),
            ComplexityControl::Measure { list_size, .. } => Some(*list_size),
        }
    }
}
