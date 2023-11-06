use crate::Context;
use async_graphql_parser::{types as ast, Positioned};

pub(crate) fn validate_object_field<'a>(
    parent_type_name: &str,
    field: &'a Positioned<ast::FieldDefinition>,
    ctx: &mut Context<'a>,
) {
    let field_name = field.node.name.node.as_str();

    // http://spec.graphql.org/draft/#sel-IAHZhCFDBDBABDl4L
    if field_name.starts_with("__") {
        let label = vec![miette::LabeledSpan::new_with_span(
            Some("here".to_owned()),
            miette::SourceSpan::new(ctx.miette_pos(field.node.name.pos), field.node.name.node.len().into()),
        )];
        ctx.push_error(miette::miette!(labels = label, r#"Field name "{parent_type_name}.{field_name}" must not begin with "__", which is reserved by GraphQL introspection."#));
    }

    super::arguments::validate_arguments(
        (parent_type_name, field.node.name.node.as_str()),
        &field.node.arguments,
        ctx,
    );
}
