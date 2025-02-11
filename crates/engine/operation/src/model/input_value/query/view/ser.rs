use schema::InputValueSet;
use walker::Walk;

use crate::QueryInputValueRecord;

use super::{QueryInputValueView, QueryOrSchemaInputValueView};

impl serde::Serialize for QueryInputValueView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let InputValueSet::SelectionSet(selection_set) = self.selection_set else {
            return self.value.serialize(serializer);
        };
        let QueryInputValueRecord::InputObject(fields) = self.value.ref_ else {
            return Err(serde::ser::Error::custom(
                "Can only select fields within an input object.",
            ));
        };
        serializer.collect_map(
            fields
                .walk(self.value.ctx)
                .filter_map(|(input_value_definition, value)| {
                    if let Some(item) = selection_set
                        .iter()
                        .find(|item| item.definition_id == input_value_definition.id)
                    {
                        if value.is_undefined() {
                            input_value_definition.default_value().map(|value| {
                                (
                                    input_value_definition.name(),
                                    QueryOrSchemaInputValueView::Schema(value.with_selection_set(&item.subselection)),
                                )
                            })
                        } else {
                            Some((
                                input_value_definition.name(),
                                QueryOrSchemaInputValueView::Query(value.with_selection_set(&item.subselection)),
                            ))
                        }
                    } else {
                        None
                    }
                }),
        )
    }
}

impl serde::Serialize for QueryOrSchemaInputValueView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            QueryOrSchemaInputValueView::Query(value) => value.serialize(serializer),
            QueryOrSchemaInputValueView::Schema(value) => value.serialize(serializer),
        }
    }
}
