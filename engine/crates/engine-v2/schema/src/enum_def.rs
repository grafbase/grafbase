use crate::{EnumDefinition, EnumValueId};

impl<'a> EnumDefinition<'a> {
    pub fn find_value_by_name(&self, name: &str) -> Option<EnumValueId> {
        self.values().find(|value| value.name() == name).map(|value| value.id())
    }
}
