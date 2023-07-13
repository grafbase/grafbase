use super::JsonMap;
use crate::{
    registry::{type_kinds::SelectionSetTarget, TypeCondition},
    Context, Error,
};
use dynaql_parser::types::Selection;

pub(super) fn project<'a, T>(
    selection: T,
    target: SelectionSetTarget<'a>,
    context: &'a Context<'a>,
) -> Result<JsonMap, Error>
where
    T: Iterator<Item = &'a Selection> + 'a,
{
    let mut map = JsonMap::new();

    for selection in selection {
        match selection {
            Selection::Field(field) => {
                let field_name = field.name.as_str();
                let meta_field = target.field(field_name).unwrap();
                let database_name = meta_field.target_field_name().to_string();

                map.insert(database_name, serde_json::Value::from(1));
            }
            Selection::FragmentSpread(fragment) => {
                let fragment = context
                    .get_fragment(fragment.fragment_name())
                    .ok_or_else(|| {
                        Error::new(format!(
                            "could not find a fragment with name '{}'",
                            fragment.fragment_name()
                        ))
                    })?;

                let target = context
                    .registry()
                    .lookup(&TypeCondition::from(fragment.type_condition().on.as_str()))?;

                let projection = project(fragment.selection(), target, context)?;
                map.extend(projection)
            }
            Selection::InlineFragment(fragment) => {
                let target = match fragment.type_condition() {
                    Some(condition) => context
                        .registry()
                        .lookup(&TypeCondition::from(condition.on.as_str()))?,
                    None => target,
                };

                let projection = project(fragment.selection(), target, context)?;
                map.extend(projection)
            }
        }
    }

    Ok(map)
}
