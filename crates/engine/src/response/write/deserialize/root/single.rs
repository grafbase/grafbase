use error::GraphqlError;
use serde::{Deserializer, de::DeserializeSeed};

use crate::response::{
    ResponseObjectRef, SeedState,
    write::deserialize::{ConcreteShapeFieldsSeed, ObjectValue},
};

pub(crate) struct RootFieldsSeed<'ctx, 'parent, 'state> {
    pub(in crate::response::write::deserialize) state: &'state SeedState<'ctx, 'parent>,
    pub(in crate::response::write::deserialize) parent_object: &'parent ResponseObjectRef,
}

impl<'de> DeserializeSeed<'de> for RootFieldsSeed<'_, '_, '_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let RootFieldsSeed { state, parent_object } = self;

        let fields_seed = {
            ConcreteShapeFieldsSeed::new(
                state,
                state.root_shape.concrete_shape(),
                parent_object.id,
                Some(parent_object.definition_id),
            )
        };

        state.reset(parent_object.path.as_slice());
        deserializer
            .deserialize_any(fields_seed)
            .map(|value| match value {
                ObjectValue::Some { fields, .. } => {
                    state.response.borrow_mut().insert_fields_update(parent_object, fields)
                }
                ObjectValue::Null => state.insert_empty_update(parent_object),
                // Errors have already been handled.
                ObjectValue::Unexpected => {
                    state.insert_propagated_empty_update(parent_object);
                }
                ObjectValue::Error(error) => {
                    state.insert_error_update(parent_object, error);
                }
            })
            .inspect_err(|err| match state.bubbling_up_deser_error.replace(true) {
                true => state.insert_propagated_empty_update(parent_object),
                false => {
                    tracing::error!(
                        "Deserialization failure of subgraph response at path '{}': {err}",
                        state.display_path()
                    );
                    state.insert_error_update(parent_object, GraphqlError::invalid_subgraph_response());
                }
            })
    }
}
