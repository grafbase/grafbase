use super::SchemaWalker;
use crate::{DefinitionWalker, ListWrapping, TypeId, Wrapping};

pub type TypeWalker<'a> = SchemaWalker<'a, TypeId>;

impl<'a> TypeWalker<'a> {
    pub fn wrapping(&self) -> Wrapping {
        self.as_ref().wrapping
    }

    pub fn inner(&self) -> DefinitionWalker<'a> {
        self.walk(self.as_ref().inner)
    }
}

impl std::fmt::Display for TypeWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in self.as_ref().wrapping.list_wrappings().rev() {
            write!(f, "[")?;
        }
        write!(f, "{}", self.inner().name())?;
        if self.as_ref().wrapping.inner_is_required() {
            write!(f, "!")?;
        }
        for wrapping in self.as_ref().wrapping.list_wrappings() {
            write!(f, "]")?;
            if wrapping == ListWrapping::RequiredList {
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
