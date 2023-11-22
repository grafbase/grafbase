use engine_parser::Pos;
use engine_value::ConstValue;

pub struct VariableDefinition {
    pub name: String,
    pub name_location: Pos,
    pub directives: Vec<()>,
    pub default_value: Option<ConstValue>,
    pub r#type: schema::Type,
}
