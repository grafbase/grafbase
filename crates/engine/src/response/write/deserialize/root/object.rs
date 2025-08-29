use error::GraphqlError;
use itertools::Itertools as _;
use serde::{Deserializer, de::DeserializeSeed};

use crate::response::{
    ResponseFieldsSortedByKey, ResponseObjectRef, SeedState,
    write::deserialize::{ConcreteShapeFieldsSeed, FieldsDeserializationResult},
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

        let (fields_id, mut response_fields) = state.response.borrow_mut().data.take_next_shared_fields();
        let offset = response_fields.len();
        let fields_seed = {
            ConcreteShapeFieldsSeed::new(
                state,
                state.root_shape.concrete_shape(),
                parent_object.id,
                Some(parent_object.definition_id),
                &mut response_fields,
            )
        };

        state.reset(parent_object.path.as_slice());
        let result = deserializer.deserialize_any(fields_seed);
        response_fields[offset..].sort_unstable_by_key(|field| field.key);
        let limit = response_fields.len() - offset;

        state
            .response
            .borrow_mut()
            .data
            .restore_shared_fields(fields_id, response_fields);

        match result {
            Ok(FieldsDeserializationResult::Some { .. }) => {
                let mut resp = state.response.borrow_mut();
                tracing::debug!(
                    "Updating object at '{}' with fields {}",
                    state.display_path(),
                    resp.data[fields_id][offset as usize..(offset + limit) as usize]
                        .iter()
                        .format_with(",", |field, f| f(&format_args!(
                            "{}",
                            &state.response_keys()[field.key]
                        )))
                        .to_string() // this panics otherwise if opentelemetry is enabled
                );
                resp.insert_fields_update(
                    parent_object,
                    ResponseFieldsSortedByKey::Slice {
                        fields_id,
                        offset: offset as u32,
                        limit: limit as u16,
                    },
                )
            }
            Ok(FieldsDeserializationResult::Null) => state.insert_empty_update(parent_object),
            Ok(FieldsDeserializationResult::Error(error)) => {
                state.insert_error_update(parent_object, [error]);
            }
            Err(err) => {
                match state.bubbling_up_deser_error.replace(true) {
                    true => state.insert_propagated_empty_update(parent_object),
                    false => {
                        tracing::error!(
                            "Deserialization failure of subgraph response at path '{}': {err}",
                            state.display_path()
                        );
                        state.insert_error_update(parent_object, [GraphqlError::invalid_subgraph_response()]);
                    }
                }
                return Err(err);
            }
        }

        Ok(())
    }
}
