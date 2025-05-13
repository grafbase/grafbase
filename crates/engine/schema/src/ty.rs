use crate::{Type, TypeRecord};

impl std::fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.wrapping.type_display(self.definition().name()).fmt(f)
    }
}

impl From<Type<'_>> for TypeRecord {
    fn from(ty: Type<'_>) -> Self {
        ty.item
    }
}

impl TypeRecord {
    pub fn is_required(&self) -> bool {
        self.wrapping.is_required()
    }

    pub fn is_list(&self) -> bool {
        self.wrapping.is_list()
    }

    pub fn list(self) -> Self {
        TypeRecord {
            wrapping: self.wrapping.list(),
            ..self
        }
    }

    pub fn list_non_null(self) -> Self {
        TypeRecord {
            wrapping: self.wrapping.list_non_null(),
            ..self
        }
    }

    pub fn non_null(self) -> Self {
        TypeRecord {
            wrapping: self.wrapping.non_null(),
            ..self
        }
    }

    pub fn without_list(self) -> Option<Self> {
        self.wrapping
            .without_list()
            .map(|wrapping| TypeRecord { wrapping, ..self })
    }
}
