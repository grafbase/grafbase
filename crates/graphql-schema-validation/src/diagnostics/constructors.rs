use crate::Context;
use async_graphql_parser::Pos;

pub(crate) fn double_underscore_name(bad_name: &str, pos: Pos, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(
        labels = vec![miette::LabeledSpan::new_with_span(
            None,
            miette::SourceSpan::new(ctx.miette_pos(pos), bad_name.len()),
        )],
        r#"Name "{bad_name}" must not begin with "__", which is reserved by GraphQL introspection."#
    ));
}

pub(crate) fn unknown_type(unknown_type: &str, location: &str, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(r#"Unknown type "{unknown_type}" in "{location}""#));
}

pub(crate) fn output_type_in_input_position(bad_type: &str, location: &str, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(
        r#"The type of "{location}" must be an input type, but got "{bad_type}", an output type."#
    ));
}

pub(crate) fn input_object_in_output_position(bad_type: &str, location: &str, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(
        r#"The type of "{location}" must be an output type, but got "{bad_type}", an input object."#
    ));
}

pub(crate) fn empty_object(bad_object: &str, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(
        "The object type {bad_object} has no fields. An object type must define one or more fields."
    ))
}

pub(crate) fn empty_interface(bad_interface: &str, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(
        "The interface {bad_interface} has no fields. An interface must define one or more fields."
    ))
}

pub(crate) fn empty_input_object(bad_input_object: &str, ctx: &mut Context<'_>) {
    ctx.push_error(miette::miette!(
        "The input object type {bad_input_object} has no fields. An input object type must define one or more fields."
    ))
}
