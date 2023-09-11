use grafbase_engine::Positioned;
use grafbase_engine_parser::types::{TypeDefinition, TypeKind};

use super::{
    auth_directive::AuthDirective,
    directive::Directive,
    model_directive::MODEL_DIRECTIVE,
    visitor::{Visitor, VisitorContext},
};
use crate::registry::add_query_search;

pub const SEARCH_DIRECTIVE: &str = "search";

pub struct SearchDirective;

impl Directive for SearchDirective {
    fn definition() -> String {
        format!(
            r#"
            directive @{SEARCH_DIRECTIVE} on OBJECT | FIELD_DEFINITION
            "#
        )
    }
}

impl<'a> Visitor<'a> for SearchDirective {
    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        let is_model = type_definition
            .node
            .directives
            .iter()
            .any(|directive| directive.is_model());
        if !is_model
            && type_definition
                .node
                .directives
                .iter()
                .any(|directive| directive.is_search())
        {
            ctx.report_error(
                vec![type_definition.pos],
                format!("The @{SEARCH_DIRECTIVE} directive can only be used on @{MODEL_DIRECTIVE} types."),
            );
        }
        if let TypeKind::Object(object) = &type_definition.node.kind {
            for field in &object.fields {
                if let Some(directive) = field.node.directives.iter().find(|directive| directive.is_search()) {
                    if !is_model {
                        ctx.report_error(
                            vec![directive.pos],
                            format!("The @{SEARCH_DIRECTIVE} directive can only be used on @{MODEL_DIRECTIVE} types."),
                        );
                    }

                    let field_base_type = field.node.ty.node.base.to_base_type_str();
                    match field_base_type {
                        "Int" | "Float" | "String" | "Email" | "PhoneNumber" | "URL" | "Date" | "DateTime"
                        | "Timestamp" | "Boolean" | "IPAddress" => (),
                        ty => ctx.report_error(
                            vec![directive.pos],
                            format!("The @{SEARCH_DIRECTIVE} directive cannot be used with the {ty} type."),
                        ),
                    }
                }
            }

            let model_auth = match AuthDirective::parse(ctx, &type_definition.node.directives, false) {
                Ok(auth) => auth,
                Err(err) => {
                    ctx.report_error(err.locations, err.message);
                    None
                }
            };
            add_query_search(ctx, &type_definition.node, &object.fields, model_auth.as_ref());
        }
    }
}
