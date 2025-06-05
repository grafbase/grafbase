use error::GraphqlError;
use runtime::extension::Data;
use serde::{
    Deserializer,
    de::{DeserializeSeed as _, IgnoredAny, SeqAccess, Unexpected, Visitor},
};
use walker::Walk;

use crate::{
    prepare::{FieldShape, SubgraphField},
    response::{
        ResponseObjectField, ResponseObjectRef, ResponseValue, ResponseValueId, SeedState,
        write::deserialize::{error::DeserError, field::FieldSeed, object::ConcreteShapeFieldsContext},
    },
};

impl<'ctx, 'parent> SeedState<'ctx, 'parent> {
    pub fn ingest_fields_guest_batched<ParentObjects>(
        &self,
        parent_objects: ParentObjects,
        batched_field_results: impl IntoIterator<
            IntoIter: ExactSizeIterator<Item = (SubgraphField<'ctx>, Result<Data, GraphqlError>)>,
            Item = (SubgraphField<'ctx>, Result<Data, GraphqlError>),
        >,
    ) where
        ParentObjects: IntoIterator<
                IntoIter: ExactSizeIterator<Item = &'parent ResponseObjectRef>,
                Item = &'parent ResponseObjectRef,
            > + Clone,
    {
        let object_shape = self.root_shape.concrete_shape();
        let batched_field_results = batched_field_results.into_iter();
        let mut batch_response_fields =
            vec![Vec::with_capacity(batched_field_results.len()); parent_objects.clone().into_iter().len()];

        for (partition_field, result) in batched_field_results {
            let field = object_shape
                .fields()
                .find(|field_shape| field_shape.as_ref().id == partition_field.id)
                .unwrap();
            let mut parents = parent_objects.clone().into_iter().zip(batch_response_fields.iter_mut());

            let err = match result {
                Ok(data) => {
                    let seed = BatchFieldsSeed {
                        state: self,
                        parents: &mut parents,
                        field,
                    };
                    let result = match &data {
                        Data::Json(bytes) => seed
                            .deserialize(&mut sonic_rs::Deserializer::from_slice(bytes))
                            .map_err(DeserError::from),
                        Data::Cbor(bytes) => seed
                            .deserialize(&mut minicbor_serde::Deserializer::new(bytes))
                            .map_err(DeserError::from),
                    };

                    match result {
                        Ok(true) => continue,
                        Ok(false) => GraphqlError::invalid_subgraph_response(),
                        Err(err) => {
                            tracing::error!(
                                "Deserialization failure of for the batch field '{}': {err}",
                                field.partition_field().definition()
                            );
                            err.into()
                        }
                    }
                }
                Err(err) => err,
            };
            write_field_error(self, field, parents, err);
        }

        let ctx = ConcreteShapeFieldsContext::new(self, object_shape);
        for (parent_object, mut fields) in parent_objects.into_iter().zip(batch_response_fields) {
            ctx.finalize_deserialized_object_fields(parent_object.id, &mut fields);
            self.response.borrow_mut().insert_fields_update(parent_object, fields)
        }
    }
}

fn write_field_error<'ctx, 'parent, 'a>(
    state: &SeedState<'ctx, 'parent>,
    field: FieldShape<'ctx>,
    mut parents: impl Iterator<Item = (&'parent ResponseObjectRef, &'a mut Vec<ResponseObjectField>)>,
    err: GraphqlError,
) {
    let key = field.key();
    if key.query_position.is_some() {
        let field = field.as_ref();
        let mut resp = state.response.borrow_mut();
        if field.wrapping.is_required() {
            let mut err = Some(err);
            for (parent_object, response_fields) in parents {
                response_fields.push(ResponseObjectField {
                    key,
                    value: ResponseValue::Unexpected,
                });
                resp.propagate_null_parent_path(&parent_object.path);
                if let Some(err) = err.take() {
                    resp.errors.push(
                        err.with_path((parent_object.path.as_slice(), key))
                            .with_location(field.id.walk(state).location()),
                    );
                }
            }
        } else {
            if let Some((parent_object, response_fields)) = parents.next() {
                response_fields.push(ResponseObjectField {
                    key,
                    value: ResponseValue::Null,
                });
                resp.errors.push(
                    err.with_path((parent_object.path.as_slice(), key))
                        .with_location(field.id.walk(state).location()),
                );
            }
            for (_, response_fields) in parents {
                response_fields.push(ResponseObjectField {
                    key,
                    value: ResponseValue::Null,
                });
            }
        }
    } else {
        for (_, response_fields) in parents {
            response_fields.push(ResponseObjectField {
                key,
                value: ResponseValue::Unexpected,
            });
        }
    }
}

struct BatchFieldsSeed<'ctx, 'parent, 'state, Parents> {
    state: &'state SeedState<'ctx, 'parent>,
    parents: Parents,
    field: FieldShape<'ctx>,
}

impl<ParentObjects> BatchFieldsSeed<'_, '_, '_, ParentObjects> {
    fn unexpected_type(&self, value: Unexpected<'_>) -> bool {
        tracing::error!(
            "invalid type: {}, expected a list for the batched field '{}'",
            value,
            self.field.partition_field().definition()
        );
        false
    }
}

impl<'ctx, 'parent, 'de, 'fields, ParentObjects> serde::de::DeserializeSeed<'de>
    for BatchFieldsSeed<'ctx, 'parent, '_, ParentObjects>
where
    ParentObjects: Iterator<Item = (&'parent ResponseObjectRef, &'fields mut Vec<ResponseObjectField>)>,
{
    type Value = bool;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'ctx, 'parent, 'de, 'fields, ParentObjects> Visitor<'de> for BatchFieldsSeed<'ctx, 'parent, '_, ParentObjects>
where
    ParentObjects: Iterator<Item = (&'parent ResponseObjectRef, &'fields mut Vec<ResponseObjectField>)>,
{
    type Value = bool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("any value?")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Self {
            field,
            state,
            parents: mut parent_objects,
        } = self;
        let key = field.key();
        let field = field.as_ref();

        while let Some((parent_object, response_fields)) = parent_objects.next() {
            state.reset(parent_object.path.as_slice());
            state.local_path_mut().push(ResponseValueId::Field {
                object_id: parent_object.id,
                key,
                nullable: field.wrapping.is_nullable(),
            });
            let value = seq.next_element_seed(FieldSeed {
                state,
                field,
                wrapping: field.wrapping.to_mutable(),
            })?;
            state.local_path_mut().pop();
            match value {
                Some(value) => {
                    state.local_path_mut().pop();
                    response_fields.push(ResponseObjectField { key, value });
                }
                None => {
                    if field.wrapping.is_required() {
                        let mut resp = state.response.borrow_mut();
                        resp.propagate_null_parent_path(&parent_object.path);
                        response_fields.push(ResponseObjectField {
                            key,
                            value: ResponseValue::Unexpected,
                        });
                        for (parent_object, response_fields) in parent_objects.by_ref() {
                            resp.propagate_null_parent_path(&parent_object.path);
                            response_fields.push(ResponseObjectField {
                                key,
                                value: ResponseValue::Unexpected,
                            });
                        }
                    } else {
                        response_fields.push(ResponseObjectField {
                            key,
                            value: ResponseValue::Null,
                        });
                        for (_, response_fields) in parent_objects.by_ref() {
                            response_fields.push(ResponseObjectField {
                                key,
                                value: ResponseValue::Null,
                            });
                        }
                    }

                    break;
                }
            }
        }

        if seq.next_element::<IgnoredAny>()?.is_some() {
            // Not adding any GraphqlError as from the client perspective we have everything.
            tracing::error!("Received more entities than expected");
        }

        Ok(true)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bool(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Signed(v)))
    }

    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Unsigned(v)))
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Other(&format!("integer {v}"))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Float(v)))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Str(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Bytes(v)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Option))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self.unexpected_type(Unexpected::Unit))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        Ok(self.unexpected_type(Unexpected::Map))
    }

    fn visit_enum<A>(self, _data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        Ok(self.unexpected_type(Unexpected::Enum))
    }
}
