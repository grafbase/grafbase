use serde_json::Value;

use super::{normalize, JsonMap};
use crate::{names::MONGODB_OUTPUT_FIELD_ID, ContextField, Error, SelectionField};

pub(super) fn project<'a>(
    ctx: &'a ContextField<'a>,
    selection: impl Iterator<Item = SelectionField<'a>> + 'a,
    target: registry_v2::MetaType<'a>,
) -> Result<JsonMap, Error> {
    let mut map = JsonMap::new();
    let selection = selection.flat_map(|selection| selection.selection_set());

    recurse(ctx, selection, target, &mut map)?;

    if !map.contains_key(MONGODB_OUTPUT_FIELD_ID) {
        map.insert(MONGODB_OUTPUT_FIELD_ID.to_string(), Value::from(1));
    }

    Ok(normalize::flatten_keys(map))
}

fn recurse<'a>(
    ctx: &ContextField<'a>,
    selection: impl Iterator<Item = SelectionField<'a>> + 'a,
    target: registry_v2::MetaType<'a>,
    output: &mut JsonMap,
) -> Result<(), Error> {
    for field in selection {
        let field_name = field.field.name.as_str();

        let meta_field = target
            .field(field_name)
            .ok_or_else(|| Error::new(format!("Field name {field_name} not found from the selection type.")))?;

        let database_name = meta_field.target_field_name().to_string();

        let inner_type = meta_field.ty().named_type();
        match inner_type.fields() {
            Some(fields) => {
                let mut inner = JsonMap::new();
                let selection = field.selection_set();

                recurse(ctx, selection, inner_type, &mut inner)?;
                output.insert(database_name, Value::Object(inner));
            }
            None => {
                output.insert(database_name, Value::from(1));
            }
        }
    }

    Ok(())
}
