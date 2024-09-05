use super::SchemaWalker;
use crate::{DefinitionWalker, ListWrapping, TypeRecord, Wrapping};

pub type Type<'a> = SchemaWalker<'a, TypeRecord>;
pub type InputTypeWalker<'a> = SchemaWalker<'a, TypeRecord>;

impl<'a> Type<'a> {
    pub fn wrapping(&self) -> Wrapping {
        self.item.wrapping
    }

    pub fn inner(&self) -> DefinitionWalker<'a> {
        self.walk(self.item.definition_id)
    }
}

impl From<Type<'_>> for TypeRecord {
    fn from(input: Type) -> Self {
        input.item
    }
}

struct Ty<'a> {
    inner: DefinitionWalker<'a>,
    wrapping: Wrapping,
}

impl<'a> From<Type<'a>> for Ty<'a> {
    fn from(input: Type<'a>) -> Self {
        Ty {
            inner: input.inner(),
            wrapping: input.wrapping(),
        }
    }
}

impl std::fmt::Display for Type<'_> {
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

impl std::fmt::Debug for SchemaWalker<'_, TypeRecord> {
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
