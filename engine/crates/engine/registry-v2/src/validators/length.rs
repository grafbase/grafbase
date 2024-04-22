#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LengthValidator {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl LengthValidator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        LengthValidator { min, max }
    }
}
