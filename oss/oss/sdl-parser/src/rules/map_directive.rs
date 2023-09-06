use super::directive::Directive;

pub struct MapDirective;

pub const MAP_DIRECTIVE_NAME: &str = "map";
pub const MAP_DIRECTIVE_NAME_ARGUMENT: &str = "name";

impl Directive for MapDirective {
    fn definition() -> String {
        format!(r#"directive @{MAP_DIRECTIVE_NAME}({MAP_DIRECTIVE_NAME_ARGUMENT}: String!) on FIELD_DEFINITION"#)
    }
}
