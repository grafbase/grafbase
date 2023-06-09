use std::fmt;

use super::TypeKind;

#[derive(Debug, Clone)]
pub struct TypeCondition {
    left: TypeKind,
    right: TypeKind,
}

impl TypeCondition {
    #[must_use]
    pub fn new(left: impl Into<TypeKind>, right: impl Into<TypeKind>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl fmt::Display for TypeCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "? {} : {}", self.left, self.right)
    }
}
