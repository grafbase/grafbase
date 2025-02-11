use super::{InputValueSet, SchemaInputValue, SchemaInputValueRecord};
use walker::Walk;

pub struct SchemaInputValueView<'a> {
    pub(super) value: SchemaInputValue<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

impl serde::Serialize for SchemaInputValueView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let InputValueSet::SelectionSet(selection_set) = self.selection_set else {
            return self.value.serialize(serializer);
        };
        let SchemaInputValueRecord::InputObject(fields) = self.value.ref_ else {
            return Err(serde::ser::Error::custom(
                "Can only select fields within an input object.",
            ));
        };
        serializer.collect_map(
            fields
                .walk(self.value.schema)
                .filter_map(|(input_value_definition, value)| {
                    if let Some(item) = selection_set
                        .iter()
                        .find(|item| item.definition_id == input_value_definition.id)
                    {
                        let value = Self {
                            value,
                            selection_set: &item.subselection,
                        };
                        Some((input_value_definition.name(), value))
                    } else {
                        None
                    }
                }),
        )
    }
}
