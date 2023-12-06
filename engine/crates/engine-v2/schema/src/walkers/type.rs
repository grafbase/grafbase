use super::SchemaWalker;
use crate::{DefinitionWalker, ListWrapping, TypeId};

pub type TypeWalker<'a> = SchemaWalker<'a, TypeId>;

impl<'a> TypeWalker<'a> {
    pub fn inner(&self) -> DefinitionWalker<'a> {
        self.walk(self.get().inner)
    }
}

impl std::fmt::Display for TypeWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in self.wrapping.list_wrapping.iter().rev() {
            write!(f, "[")?;
        }
        write!(f, "{}", self.inner().name())?;
        if self.wrapping.inner_is_required {
            write!(f, "!")?;
        }
        for wrapping in &self.wrapping.list_wrapping {
            write!(f, "]")?;
            if *wrapping == ListWrapping::RequiredList {
                write!(f, "!")?;
            }
        }
        Ok(())
    }
}

impl<'a> std::fmt::Debug for TypeWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type")
            .field("name", &self.to_string())
            .field("inner", &self.inner())
            .finish()
    }
}
