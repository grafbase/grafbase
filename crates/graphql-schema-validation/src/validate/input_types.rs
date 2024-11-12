use crate::context::Context;
use async_graphql_parser::{types::TypeKind, Pos};

#[must_use]
pub(crate) enum ValidateInputTypeResult {
    Ok,
    UnknownType,
    NotAnInputType,
}

pub(crate) fn validate_input_type(type_name: &str, _pos: Pos, ctx: &mut Context<'_>) -> ValidateInputTypeResult {
    if super::is_builtin_scalar(type_name) {
        return ValidateInputTypeResult::Ok;
    }

    let Some(definition) = ctx.definition_names.get(type_name) else {
        return ValidateInputTypeResult::UnknownType;
    };

    match &definition.node.kind {
        TypeKind::Scalar | TypeKind::Enum(_) | TypeKind::InputObject(_) => ValidateInputTypeResult::Ok,
        TypeKind::Object(_) | TypeKind::Interface(_) | TypeKind::Union(_) => ValidateInputTypeResult::NotAnInputType,
    }
}
