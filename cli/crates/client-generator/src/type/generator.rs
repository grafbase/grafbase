use std::fmt;

use crate::common::Identifier;

use super::StaticType;

#[derive(Clone, Debug)]
pub struct TypeGenerator<'a> {
    param: Identifier<'a>,
    source: StaticType<'a>,
}

#[allow(dead_code)]
impl<'a> TypeGenerator<'a> {
    #[must_use]
    pub fn new(param: impl Into<Identifier<'a>>, source: StaticType<'a>) -> Self {
        Self {
            param: param.into(),
            source,
        }
    }
}

impl<'a> fmt::Display for TypeGenerator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in {}", self.param, self.source)
    }
}
