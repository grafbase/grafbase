use std::fmt;

use crate::common::Identifier;

use super::StaticType;

#[derive(Debug, Clone)]
pub struct TypeGenerator {
    param: Identifier,
    source: StaticType,
}

impl TypeGenerator {
    #[must_use]
    pub fn new(param: impl Into<Identifier>, source: StaticType) -> Self {
        Self {
            param: param.into(),
            source,
        }
    }
}

impl fmt::Display for TypeGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in {}", self.param, self.source)
    }
}
