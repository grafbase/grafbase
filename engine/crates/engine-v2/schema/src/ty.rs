use wrapping::ListWrapping;

use crate::{Type, TypeRecord};

impl TypeRecord {
    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        Self {
            wrapping: self.wrapping.wrapped_by(list_wrapping),
            ..self
        }
    }

    /// Determines whether a variable is compatible with the expected type
    pub fn is_compatible_with(&self, other: TypeRecord) -> bool {
        self.definition_id == other.definition_id
            // if not a list, the current type can be coerced into the proper list wrapping.
            && (!self.wrapping.is_list()
                || self.wrapping.list_wrappings().len() == other.wrapping.list_wrappings().len())
            && (other.wrapping.is_nullable() || self.wrapping.is_required())
    }
}

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.wrapping.list_wrappings().len() {
            write!(f, "[")?;
        }
        write!(f, "{}", self.definition().name())?;
        if self.wrapping.inner_is_required() {
            write!(f, "!")?;
        }
        for wrapping in self.wrapping.list_wrappings() {
            write!(f, "]")?;
            if wrapping == ListWrapping::RequiredList {
                write!(f, "!")?;
            }
        }
        Ok(())
    }
}

impl From<Type<'_>> for TypeRecord {
    fn from(ty: Type<'_>) -> Self {
        ty.item
    }
}
