use super::SchemaWalker;
use crate::{EnumId, EnumValue};

pub type EnumWalker<'a> = SchemaWalker<'a, EnumId>;

impl<'a> EnumWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.r#enum(self.schema, self.inner)
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description.map(|id| self.schema[id].as_str())
    }

    pub fn values(&self) -> impl Iterator<Item = &'a EnumValue> + 'a {
        self.schema[self.inner].values.iter()
    }
}

impl<'a> std::fmt::Debug for EnumWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enum")
            .field("id", &usize::from(self.inner))
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "values",
                &self.values().map(|value| &self.schema[value.name]).collect::<Vec<_>>(),
            )
            .finish()
    }
}
