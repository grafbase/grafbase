use std::fmt;

use super::TypeKind;

#[derive(Clone)]
pub struct TypeCondition<'a> {
    left: TypeKind<'a>,
    right: TypeKind<'a>,
}

impl<'a> TypeCondition<'a> {
    #[must_use]
    pub fn new(left: impl Into<TypeKind<'a>>, right: impl Into<TypeKind<'a>>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl<'a> fmt::Display for TypeCondition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "? {} : {}", self.left, self.right)
    }
}
