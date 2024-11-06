use walker::Walk;

use crate::operation::QueryInputValueRecord;

use super::{QueryInputValueView, QueryOrSchemaInputValueView};

impl<'a> serde::Serialize for QueryInputValueView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Composition guarantees a proper InputValueSet, so if the selection set is empty it means
        // we're serializing a scalar.
        if self.selection_set.is_empty() {
            return self.value.serialize(serializer);
        }
        let QueryInputValueRecord::InputObject(fields) = self.value.ref_ else {
            return Err(serde::ser::Error::custom(
                "Can only select fields within an input object.",
            ));
        };
        serializer.collect_map(
            fields
                .walk(self.value.ctx)
                .filter_map(|(input_value_definition, value)| {
                    if let Some(item) = self
                        .selection_set
                        .iter()
                        .find(|item| item.id == input_value_definition.id())
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

impl<'a> serde::Serialize for QueryOrSchemaInputValueView<'a> {
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
