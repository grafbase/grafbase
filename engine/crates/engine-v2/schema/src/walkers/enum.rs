use super::SchemaWalker;
use crate::{EnumId, EnumValue};

pub type EnumWalker<'a> = SchemaWalker<'a, EnumId>;

impl<'a> EnumWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.r#enum(self.schema, self.item)
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = &'a EnumValue> + 'a {
        self.schema[self.item].values.iter()
    }
}

impl<'a> std::fmt::Debug for EnumWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enum")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field(
                "values",
                &self.values().map(|value| &self.schema[value.name]).collect::<Vec<_>>(),
            )
            .finish()
    }
}
