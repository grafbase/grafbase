use serde_json::Value;

use super::JsonMap;
use crate::{
    registry::{type_kinds::SelectionSetTarget, variables::VariableResolveDefinition},
    Context, ServerResult,
};
use std::sync::OnceLock;

pub(super) fn by(target: SelectionSetTarget<'_>, ctx: &Context<'_>) -> ServerResult<Value> {
    static BY_FILTER: OnceLock<VariableResolveDefinition> = OnceLock::new();

    let resolve_definition =
        BY_FILTER.get_or_init(|| VariableResolveDefinition::InputTypeName("by".to_string()));

    let map: JsonMap = resolve_definition.resolve(ctx, Option::<Value>::None)?;

    Ok(Value::Object(normalize(target, map)))
}

fn normalize(target: SelectionSetTarget<'_>, map: JsonMap) -> JsonMap {
    let mut result = JsonMap::new();

    for (key, value) in map {
        let meta_field = target.field(&key).unwrap();
        let database_name = meta_field.target_field_name().to_string();

        result.insert(database_name, value);
    }

    result
}
