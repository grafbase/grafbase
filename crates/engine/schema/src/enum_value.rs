use crate::EnumValue;

impl EnumValue<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_enum_values[self.id]
    }
}
