use super::SchemaWalker;
use crate::{EnumId, EnumValueId};

pub type EnumWalker<'a> = SchemaWalker<'a, EnumId>;
pub type EnumValueWalker<'a> = SchemaWalker<'a, EnumValueId>;

impl<'a> EnumWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.r#enum(self.schema, self.item)
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = EnumValueWalker<'a>> + 'a {
        let walker: SchemaWalker<'a> = self.walk(());
        self.schema[self.item].values.iter().map(move |id| walker.walk(id))
    }
}

impl<'a> EnumValueWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.enum_value(self.schema, self.item)
    }
}

impl<'a> std::fmt::Debug for EnumWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Enum")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field("values", &self.values().map(|value| value.name()).collect::<Vec<_>>())
            .finish()
    }
}
