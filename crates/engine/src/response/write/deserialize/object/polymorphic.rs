use std::{collections::VecDeque, fmt};

use schema::TypeDefinition;
use serde::{
    Deserializer,
    de::{DeserializeSeed, IgnoredAny, MapAccess, Unexpected, Visitor},
};
use walker::Walk;

use crate::{
    prepare::{FieldShapeRecord, ObjectIdentifier, PolymorphicShapeId, PolymorphicShapeRecord},
    response::{
        GraphqlError, ResponseObject, ResponseValue,
        write::deserialize::{SeedContext, key::Key},
    },
};

use super::concrete::ConcreteShapeSeed;

pub(crate) struct PolymorphicShapeSeed<'ctx, 'seed> {
    ctx: &'seed SeedContext<'ctx>,
    parent_field: &'ctx FieldShapeRecord,
    is_required: bool,
    shape: &'ctx PolymorphicShapeRecord,
}

impl<'ctx, 'seed> PolymorphicShapeSeed<'ctx, 'seed> {
    pub fn new(
        ctx: &'seed SeedContext<'ctx>,
        parent_field: &'ctx FieldShapeRecord,
        is_required: bool,
        shape_id: PolymorphicShapeId,
    ) -> Self {
        let polymorphic = shape_id.walk(ctx);
        Self {
            ctx,
            parent_field,
            is_required,
            shape: polymorphic.as_ref(),
        }
    }
}

impl<'de> DeserializeSeed<'de> for PolymorphicShapeSeed<'_, '_> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl PolymorphicShapeSeed<'_, '_> {
    fn unexpected_type(&self, value: Unexpected<'_>) -> <Self as Visitor<'_>>::Value {
        tracing::error!(
            "invalid type: {}, expected an object at path '{}'",
            value,
            self.ctx.display_path()
        );

        if self.parent_field.key.query_position.is_some() {
            let mut resp = self.ctx.response.borrow_mut();
            let path = self.ctx.path();
            // If not required, we don't need to propagate as Unexpected is equivalent to
            // null for users.
            if self.is_required {
                resp.propagate_null(&path);
            }
            resp.errors.push(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path)
                    .with_location(self.parent_field.id.walk(self.ctx).location()),
            );
        }

        ResponseValue::Unexpected
    }
}

impl<'de> Visitor<'de> for PolymorphicShapeSeed<'_, '_> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("any value?")
    }

    // later we could also support visit_struct by using the schema as the reference structure.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let schema = self.ctx.schema;
        let mut content = VecDeque::<(Key<'_>, serde_value::Value)>::new();
        while let Some(key) = map.next_key::<Key<'de>>()? {
            if key.as_ref() == "__typename" {
                let value = map.next_value::<Key<'_>>()?;
                let typename = value.as_ref();

                let Some(TypeDefinition::Object(object_definition)) = schema.type_definition_by_name(typename) else {
                    tracing::error!(
                        "Couldn't determine the object type from __typename at path '{}'",
                        self.ctx.display_path()
                    );
                    break;
                };
                let object_definition_id = object_definition.id;

                // Try to find the matching concrete shape.
                return if let Ok(i) = self
                    .shape
                    .possibilities
                    .binary_search_by(|(id, _)| id.cmp(&object_definition_id))
                {
                    let (_, shape_id) = self.shape.possibilities[i];
                    ConcreteShapeSeed::new_with_known_object_definition_id(
                        self.ctx,
                        self.parent_field,
                        self.is_required,
                        shape_id,
                        object_definition_id,
                    )
                    .visit_map(ChainedMapAcces::new(content, map))
                } else if let Some(shape_id) = self.shape.fallback {
                    // We're falling back on the fallback shape. It may or may not need the object
                    // definition id. ConcreteShapeSeed relies on encountering "__typename" field
                    // like we do to detect the object definition id, but we've already
                    // deserialized it. So we have to provide the object definition id to the
                    // concrete shape seed if needed, as it won't be able to do it.
                    match shape_id.walk(self.ctx).identifier {
                        ObjectIdentifier::UnionTypename(id)
                            if !id.walk(self.ctx.schema).has_member(object_definition_id) =>
                        {
                            tracing::error!(
                                "Unexpected object '{}' for union '{}' at path '{}'",
                                object_definition.name(),
                                id.walk(self.ctx.schema).name(),
                                self.ctx.display_path()
                            );
                            break;
                        }
                        ObjectIdentifier::InterfaceTypename(id)
                            if !id.walk(self.ctx.schema).has_implementor(object_definition_id) =>
                        {
                            tracing::error!(
                                "Unexpected object '{}' for interface '{}' at path '{}'",
                                object_definition.name(),
                                id.walk(self.ctx.schema).name(),
                                self.ctx.display_path()
                            );
                            break;
                        }
                        _ => {}
                    };

                    ConcreteShapeSeed::new_with_known_object_definition_id(
                        self.ctx,
                        self.parent_field,
                        self.is_required,
                        shape_id,
                        object_definition_id,
                    )
                    .visit_map(ChainedMapAcces::new(content, map))
                } else {
                    // If the __typename doesn't match any of the possibilities nor do we have a
                    // fallback, there is no field to retrieve.

                    // Try discarding the next value, we might be able to use other parts of
                    // the response.
                    while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}

                    // Adding empty object instead
                    Ok(self
                        .ctx
                        .response
                        .borrow_mut()
                        .data
                        .push_object(ResponseObject::new(Some(object_definition_id), Vec::new()))
                        .into())
                };
            }
            // keeping the fields until we find the actual __typename.
            content.push_back((key, map.next_value()?));
        }

        // Try discarding the rest of the map, we might be able to use other parts of
        // the response.
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}

        if self.parent_field.key.query_position.is_some() {
            let mut resp = self.ctx.response.borrow_mut();
            let path = self.ctx.path();
            // If not required, we don't need to propagate as Unexpected is equivalent to
            // null for users.
            if self.is_required {
                resp.propagate_null(&path);
            }
            resp.errors.push(
                GraphqlError::invalid_subgraph_response()
                    .with_path(path)
                    .with_location(self.parent_field.id.walk(self.ctx).location()),
            );
        }

        Ok(ResponseValue::Unexpected)
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
        if self.is_required {
            Ok(self.unexpected_type(Unexpected::Option))
        } else {
            Ok(ResponseValue::Null)
        }
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
        if self.is_required {
            Ok(self.unexpected_type(Unexpected::Unit))
        } else {
            Ok(ResponseValue::Null)
        }
    }

    fn visit_newtype_struct<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // newtype_struct are used by custom deserializers to indicate that an error happened, but
        // was already treated.
        Ok(ResponseValue::Unexpected)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        // Try discarding the rest of the list, we might be able to use other parts of
        // the response.
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(self.unexpected_type(Unexpected::Seq))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        let _ = data.variant::<IgnoredAny>()?;
        Ok(self.unexpected_type(Unexpected::Enum))
    }
}

struct ChainedMapAcces<'de, A> {
    before: VecDeque<(Key<'de>, serde_value::Value)>,
    next_value: Option<serde_value::Value>,
    after: A,
}

impl<'de, A> ChainedMapAcces<'de, A> {
    fn new(before: VecDeque<(Key<'de>, serde_value::Value)>, after: A) -> Self {
        Self {
            before,
            next_value: None,
            after,
        }
    }
}

impl<'de, A> MapAccess<'de> for ChainedMapAcces<'de, A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some((key, value)) = self.before.pop_front() {
            self.next_value = Some(value);
            seed.deserialize(serde_value::ValueDeserializer::new(serde_value::Value::String(
                key.into_string(),
            )))
            .map(Some)
        } else {
            self.after.next_key_seed(seed)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(value) = self.next_value.take() {
            seed.deserialize(serde_value::ValueDeserializer::new(value))
        } else {
            self.after.next_value_seed(seed)
        }
    }
}
