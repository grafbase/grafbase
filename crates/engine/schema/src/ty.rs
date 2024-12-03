use crate::{Type, TypeRecord};

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.wrapping.write_type_string(self.definition().name(), f)
    }
}

impl From<Type<'_>> for TypeRecord {
    fn from(ty: Type<'_>) -> Self {
        ty.item
    }
}
