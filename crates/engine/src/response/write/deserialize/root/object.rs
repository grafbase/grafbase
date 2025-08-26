use error::GraphqlError;
use itertools::Itertools as _;
use serde::{Deserializer, de::DeserializeSeed};

use crate::response::{
    ResponseObjectRef, SeedState,
    write::deserialize::{ConcreteShapeFieldsSeed, ObjectFields},
};

impl<'ctx, 'parent> SeedState<'ctx, 'parent> {
    pub fn parent_seed(&self, parent_object: &'parent ResponseObjectRef) -> RootObjectSeed<'ctx, 'parent, '_> {
        RootObjectSeed {
            state: self,
            parent_object,
        }
    }
}

pub(crate) struct RootObjectSeed<'ctx, 'parent, 'state> {
    state: &'state SeedState<'ctx, 'parent>,
    parent_object: &'parent ResponseObjectRef,
}

impl<'de> DeserializeSeed<'de> for RootObjectSeed<'_, '_, '_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let RootObjectSeed { state, parent_object } = self;

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
                ObjectFields::Some { fields, .. } => {
                    tracing::debug!(
                        "Updating object at '{}' with fields {}",
                        state.display_path(),
                        fields
                            .iter()
                            .format_with(",", |field, f| f(&format_args!(
                                "{}",
                                &state.response_keys()[field.key]
                            )))
                            .to_string() // this panics otherwise if opentelemetry is enabled
                    );
                    state.response.borrow_mut().insert_fields_update(parent_object, fields)
                }
                ObjectFields::Null => state.insert_empty_update(parent_object),
                ObjectFields::Error(error) => {
                    state.insert_error_update(parent_object, [error]);
                }
            })
            .inspect_err(|err| match state.bubbling_up_deser_error.replace(true) {
                true => state.insert_propagated_empty_update(parent_object),
                false => {
                    tracing::error!(
                        "Deserialization failure of subgraph response at path '{}': {err}",
                        state.display_path()
                    );
                    state.insert_error_update(parent_object, [GraphqlError::invalid_subgraph_response()]);
                }
            })
    }
}
