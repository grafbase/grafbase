use std::marker::PhantomData;

use error::GraphqlError;
use runtime::extension::Data;
use serde::{
    Deserializer,
    de::{MapAccess, Visitor},
    forward_to_deserialize_any,
};

use crate::prepare::SubgraphField;

use super::SeedContext;

pub(super) struct EntityFields<'de, 'ctx, 'seed> {
    pub ctx: &'seed SeedContext<'ctx>,
    pub fields: &'de mut [(SubgraphField<'ctx>, Result<Data, GraphqlError>)],
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DeserError {
    #[error(transparent)]
    Json(#[from] sonic_rs::Error),
    #[error(transparent)]
    Cbor(#[from] minicbor_serde::error::DecodeError),
    #[error("{0}")]
    Message(String),
}

impl serde::de::Error for DeserError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        DeserError::Message(msg.to_string())
    }
}

impl<'de> Deserializer<'de> for EntityFields<'de, '_, '_> {
    type Error = DeserError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(EntityFieldsMapAccess {
            ctx: self.ctx,
            fields: self.fields,
            index: 0,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char string
        bytes byte_buf unit unit_struct seq tuple newtype_struct str
        tuple_struct struct enum identifier
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct EntityFieldsMapAccess<'de, 'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    fields: &'de [(SubgraphField<'ctx>, Result<Data, GraphqlError>)],
    index: usize,
}

impl<'de> MapAccess<'de> for EntityFieldsMapAccess<'de, '_, '_> {
    type Error = DeserError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        while let Some((field, data)) = self.fields.get(self.index) {
            if let Err(err) = data {
                self.index += 1;
                if field.query_position().is_some() {
                    let mut resp = self.ctx.subgraph_response.borrow_mut();
                    let path = self.ctx.path();
                    // If not required, we don't need to propagate as Unexpected is equivalent to
                    // null for users.
                    if field.definition().ty().wrapping.is_required() {
                        resp.propagate_null(&path);
                    }
                    // FIXME: remove Clone...
                    resp.push_error(
                        err.clone()
                            .with_path((path, field.response_key()))
                            .with_location(field.location()),
                    );
                }

                continue;
            }
            return seed
                .deserialize(Key {
                    key: field.subgraph_response_key_str(),
                    _phantom: PhantomData::<Self::Error>,
                })
                .map(Some);
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let (_, result) = self
            .fields
            .get(self.index)
            .expect("Should have checked with next_key_seed");
        self.index += 1;
        match result {
            Ok(data) => match data {
                Data::Json(bytes) => seed
                    .deserialize(&mut sonic_rs::Deserializer::from_slice(bytes))
                    .map_err(Into::into),
                Data::Cbor(bytes) => {
                    let mut de = minicbor_serde::Deserializer::<'de>::new(bytes);
                    seed.deserialize(&mut de).map_err(Into::into)
                }
            },
            Err(_) => {
                unreachable!("Error should have been handled in next_key_seed");
            }
        }
    }
}

struct Key<'k, Error> {
    key: &'k str,
    _phantom: PhantomData<Error>,
}

impl<'de, Error: serde::de::Error> Deserializer<'de> for Key<'de, Error> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.key)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.key)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct enum identifier
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}
