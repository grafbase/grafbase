use crate::{EnumDefinition, EnumValue};

impl<'a> EnumDefinition<'a> {
    pub fn find_value_by_name(&self, name: &str) -> Option<EnumValue<'a>> {
        self.values().find(|value| value.name() == name)
    }

    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible.enum_definitions[self.id]
    }
}
