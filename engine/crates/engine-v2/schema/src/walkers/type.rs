use super::SchemaWalker;
use crate::{DefinitionWalker, ListWrapping, TypeId};

pub type TypeWalker<'a> = SchemaWalker<'a, TypeId>;

impl<'a> TypeWalker<'a> {
    pub fn name(&self) -> String {
        let mut name = self.inner().name().to_string();
        if self.wrapping.inner_is_required {
            name.push('!');
        }
        for list_wrapping in &self.wrapping.list_wrapping {
            name = match list_wrapping {
                ListWrapping::RequiredList => format!("[{name}]!"),
                ListWrapping::NullableList => format!("[{name}]"),
            }
        }
        name
    }

    pub fn inner(&self) -> DefinitionWalker<'a> {
        self.walk(self.inner)
    }
}

impl<'a> std::fmt::Debug for TypeWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<TypeWalker<'_>>())
            .field("name", &self.name())
            .field("inner", &self.inner())
            .finish()
    }
}
