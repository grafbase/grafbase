use super::SchemaWalker;
use crate::ScalarDefinitionId;

pub type ScalarWalker<'a> = SchemaWalker<'a, ScalarDefinitionId>;

impl<'a> ScalarWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.scalar(self.schema, self.item)
    }
}

impl<'a> std::fmt::Debug for ScalarWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scalar")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .finish()
    }
}
