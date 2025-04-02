use crate::EnumValue;

impl EnumValue<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_enum_values[self.id]
    }
}

impl std::fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumValue")
            .field("name", &self.name())
            .field("parent_enum", &self.parent_enum().name())
            .field("description", &self.description())
            .field("directives", &self.directives())
            .finish()
    }
}
