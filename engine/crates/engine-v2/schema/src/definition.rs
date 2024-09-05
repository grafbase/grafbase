use crate::Definition;

impl<'a> Definition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Definition::Enum(item) => item.name(),
            Definition::InputObject(item) => item.name(),
            Definition::Interface(item) => item.name(),
            Definition::Object(item) => item.name(),
            Definition::Scalar(item) => item.name(),
            Definition::Union(item) => item.name(),
        }
    }
}
