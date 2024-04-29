mod length;

pub use length::LengthValidator;

// Wrap Validators up in an enum to avoid having to box the context data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum DynValidator {
    Length(LengthValidator),
}

impl DynValidator {
    pub fn length(min: Option<usize>, max: Option<usize>) -> Self {
        Self::Length(LengthValidator::new(min, max))
    }
}
