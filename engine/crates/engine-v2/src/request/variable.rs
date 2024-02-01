use engine_value::ConstValue;

use super::Location;

pub struct VariableDefinition {
    pub name: String,
    pub name_location: Location,
    pub directives: Vec<()>,
    pub default_value: Option<ConstValue>,
    pub r#type: schema::Type,
}
