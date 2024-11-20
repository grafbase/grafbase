use crate::EnumValue;

impl<'a> EnumValue<'a> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_enum_values[self.id]
    }
}
