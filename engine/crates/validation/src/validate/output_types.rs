use crate::context::Context;
use async_graphql_parser::{types::TypeKind, Pos};

#[must_use]
pub(crate) enum ValidateOutputTypeResult {
    Ok,
    UnknownType,
    InputObject,
}

pub(crate) fn validate_output_type(name: &str, _pos: Pos, ctx: &mut Context<'_>) -> ValidateOutputTypeResult {
    if super::is_builtin_scalar(name) {
        return ValidateOutputTypeResult::Ok;
    }

    let Some(definition) = ctx.definition_names.get(name) else {
        return ValidateOutputTypeResult::UnknownType;
    };

    match definition.node.kind {
        TypeKind::Scalar | TypeKind::Object(_) | TypeKind::Interface(_) | TypeKind::Union(_) | TypeKind::Enum(_) => {
            ValidateOutputTypeResult::Ok
        }
        TypeKind::InputObject(_) => ValidateOutputTypeResult::InputObject,
    }
}
