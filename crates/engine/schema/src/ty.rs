use wrapping::ListWrapping;

use crate::{Type, TypeRecord};

impl TypeRecord {
    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        Self {
            wrapping: self.wrapping.wrapped_by(list_wrapping),
            ..self
        }
    }
}

impl<'a> Type<'a> {
    pub fn wrapped_by(self, list_wrapping: ListWrapping) -> Self {
        Self {
            item: self.item.wrapped_by(list_wrapping),
            ..self
        }
    }

    pub fn pop_list_wrapping(&mut self) -> Option<ListWrapping> {
        self.item.wrapping.pop_list_wrapping()
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
