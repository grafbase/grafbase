use super::SchemaWalker;
use crate::{DefinitionWalker, ListWrapping, Type, Wrapping};

pub type TypeWalker<'a> = SchemaWalker<'a, Type>;
pub type InputTypeWalker<'a> = SchemaWalker<'a, Type>;

impl<'a> TypeWalker<'a> {
    pub fn wrapping(&self) -> Wrapping {
        self.item.wrapping
    }

    pub fn inner(&self) -> DefinitionWalker<'a> {
        self.walk(self.item.inner)
    }
}

impl From<TypeWalker<'_>> for Type {
    fn from(input: TypeWalker) -> Self {
        input.item
    }
}

struct Ty<'a> {
    inner: DefinitionWalker<'a>,
    wrapping: Wrapping,
}

impl<'a> From<TypeWalker<'a>> for Ty<'a> {
    fn from(input: TypeWalker<'a>) -> Self {
        Ty {
            inner: input.inner(),
            wrapping: input.wrapping(),
        }
    }
}

impl std::fmt::Display for TypeWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ty::from(*self).fmt(f)
    }
}

impl std::fmt::Display for Ty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.wrapping.list_wrappings().len() {
            write!(f, "[")?;
        }
        write!(f, "{}", self.inner.name())?;
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

impl std::fmt::Debug for SchemaWalker<'_, Type> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ty::from(*self).fmt(f)
    }
}

impl std::fmt::Debug for Ty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type")
            .field("name", &self.to_string())
            .field("inner", &self.inner)
            .finish()
    }
}
