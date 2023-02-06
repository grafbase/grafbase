use super::{
    directive::Directive,
    visitor::{Visitor, VisitorContext},
};
use dynaql::Positioned;
use dynaql_parser::types::{FieldDefinition, TypeDefinition};
use dynaql_value::ConstValue;

pub const RESOLVER_DIRECTIVE: &str = "resolver";
pub const NAME_ARGUMENT: &str = "value";

pub struct ResolverDirective;

impl ResolverDirective {
    #[allow(dead_code)]
    pub fn resolver_name(field: &FieldDefinition) -> Option<&str> {
        field
            .directives
            .iter()
            .find(|directive| directive.node.name.node == RESOLVER_DIRECTIVE)
            .and_then(|directive| directive.node.get_argument(NAME_ARGUMENT))
            .and_then(|value| match &value.node {
                ConstValue::String(resolver_name) => Some(resolver_name.as_str()),
                _ => None,
            })
    }
}

impl<'a> Visitor<'a> for ResolverDirective {
    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
        if let Some(directive) = field
            .node
            .directives
            .iter()
            .find(|d| d.node.name.node == RESOLVER_DIRECTIVE)
        {
            if let Ok(mut arguments) = super::directive::extract_arguments(ctx, directive, &[&[NAME_ARGUMENT]], None) {
                if let ConstValue::String(_resolver_name) = arguments.remove(NAME_ARGUMENT).unwrap() {
                    // OK.
                } else {
                    ctx.report_error(
                        vec![directive.pos],
                        "The @{RESOLVER_DIRECTIVE} directive expects the `{name}` argument to be a string".to_string(),
                    );
                }
            }
        }
    }
}

impl Directive for ResolverDirective {
    fn definition() -> String {
        r#"
        directive @resolver(name: String) on FIELD_DEFINITION
        "#
        .to_string()
    }
}
