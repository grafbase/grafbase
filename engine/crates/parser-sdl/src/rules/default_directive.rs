use engine_parser::types::FieldDefinition;
use engine_value::ConstValue;

use super::{default_directive_types::VALUE_ARGUMENT, directive::Directive};

pub const DEFAULT_DIRECTIVE: &str = "default";

pub struct DefaultDirective;

impl DefaultDirective {
    pub fn default_value_of(field: &FieldDefinition) -> Option<ConstValue> {
        field
            .directives
            .iter()
            .find(|directive| directive.node.name.node == DEFAULT_DIRECTIVE)
            .and_then(|directive| directive.node.get_argument(VALUE_ARGUMENT))
            .cloned()
            .map(|value| value.node)
    }
}

impl Directive for DefaultDirective {
    fn definition() -> String {
        r"
        directive @default(value: String) on FIELD_DEFINITION
        "
        .to_string()
    }
}
