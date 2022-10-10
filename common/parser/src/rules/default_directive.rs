use super::visitor::Visitor;

pub const DEFAULT_DIRECTIVE: &str = "default";

pub struct DefaultDirective;

impl<'a> Visitor<'a> for DefaultDirective {
    fn directives(&self) -> String {
        r#"
        directive @default(value: String) on FIELD_DEFINITION
        "#
        .to_string()
    }
}
