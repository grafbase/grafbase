use crate::{DeprecatedDirective, EnumValue};

impl EnumValue<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible.enum_values[self.id]
    }

    pub fn has_deprecated(&self) -> Option<DeprecatedDirective<'_>> {
        self.directives().find_map(|directive| directive.as_deprecated())
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
