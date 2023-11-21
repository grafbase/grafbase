use super::SchemaWalker;
use crate::ScalarId;

pub type ScalarWalker<'a> = SchemaWalker<'a, ScalarId>;

impl<'a> ScalarWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.scalar(self.id)
    }

    pub fn specified_by_url(&self) -> Option<&'a str> {
        self.specified_by_url.map(|id| self.schema[id].as_str())
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }
}

impl<'a> std::fmt::Debug for ScalarWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<ScalarWalker<'_>>())
            .field("id", &usize::from(self.id))
            .field("name", &self.name())
            .field("description", &self.description())
            .field("specified_by_url", &self.specified_by_url())
            .finish()
    }
}
