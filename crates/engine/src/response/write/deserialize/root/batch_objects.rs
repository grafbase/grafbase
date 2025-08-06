use error::GraphqlError;
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, SeqAccess, Visitor},
};

use crate::response::{ResponseObjectRef, SeedState};

impl<'ctx, 'parent> SeedState<'ctx, 'parent> {
    pub fn parent_list_seed<ParentObjects>(
        &self,
        parent_objects: ParentObjects,
    ) -> BatchRootObjectsSeed<'ctx, 'parent, '_, ParentObjects>
    where
        ParentObjects: IntoIterator<
                IntoIter: ExactSizeIterator<Item = &'parent ResponseObjectRef>,
                Item = &'parent ResponseObjectRef,
            >,
    {
        BatchRootObjectsSeed {
            state: self,
            parent_objects,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BatchRootObjectsSeed<'ctx, 'parent, 'state, ParentObjects> {
    state: &'state SeedState<'ctx, 'parent>,
    parent_objects: ParentObjects,
}

impl<'de, 'parent, ParentObjects> DeserializeSeed<'de> for BatchRootObjectsSeed<'_, 'parent, '_, ParentObjects>
where
    ParentObjects:
        IntoIterator<IntoIter: ExactSizeIterator<Item = &'parent ResponseObjectRef>, Item = &'parent ResponseObjectRef>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de, 'parent, ParentObjects> Visitor<'de> for BatchRootObjectsSeed<'_, 'parent, '_, ParentObjects>
where
    ParentObjects:
        IntoIterator<IntoIter: ExactSizeIterator<Item = &'parent ResponseObjectRef>, Item = &'parent ResponseObjectRef>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an entities list")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_none()
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.deserialize(serde_json::Value::Array(Vec::new()))
            .expect("Deserializer never fails");
        Ok(())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self { state, parent_objects } = self;
        let mut result = Ok(());
        let mut parent_objects = parent_objects.into_iter();
        for parent_object in parent_objects.by_ref() {
            let seed = state.parent_seed(parent_object);
            match seq.next_element_seed(seed) {
                Ok(Some(())) => (),
                Ok(None) => {
                    tracing::error!("Received less entities than expected");
                    state.insert_error_update(parent_object, [GraphqlError::invalid_subgraph_response()]);

                    break;
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

                    result = Err(err);
                    break;
                }
            }
        }

        if parent_objects.len() > 0 {
            state.insert_empty_updates(parent_objects);
        }

        // If de-serialization didn't fail, we finish consuming the sequence if there is anything
        // left.
        if result.is_ok() && seq.next_element::<IgnoredAny>()?.is_some() {
            // Not adding any GraphqlError as from the client perspective we have everything.
            tracing::error!("Received more entities than expected");

            // Try discarding the rest of the list, we might be able to use other parts of
            // the response.
            while seq.next_element::<IgnoredAny>().unwrap_or_default().is_some() {}
        }

        result
    }
}
