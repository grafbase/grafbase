use indexmap::IndexMap;
use serde_json::Value;

use super::{normalize, JsonMap};
use crate::{
    names::MONGODB_OUTPUT_FIELD_ID,
    registry::{MetaField, MetaType},
    ContextExt, ContextField, Error, SelectionField,
};

pub(super) fn project<'a>(
    ctx: &'a ContextField<'a>,
    selection: impl Iterator<Item = SelectionField<'a>> + 'a,
    target: &IndexMap<String, MetaField>,
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
    target: &IndexMap<String, MetaField>,
    output: &mut JsonMap,
) -> Result<(), Error> {
    for field in selection {
        let field_name = field.field.name.as_str();

        let meta_field = target
            .get(field_name)
            .ok_or_else(|| Error::new(format!("Field name {field_name} not found from the selection type.")))?;

        let database_name = meta_field.target_field_name().to_string();

        match ctx.get_type(meta_field.ty.base_type_name()).and_then(MetaType::fields) {
            Some(fields) => {
                let mut inner = JsonMap::new();
                let selection = field.selection_set();

                recurse(ctx, selection, fields, &mut inner)?;
                output.insert(database_name, Value::Object(inner));
            }
            None => {
                output.insert(database_name, Value::from(1));
            }
        }
    }

    Ok(())
}
