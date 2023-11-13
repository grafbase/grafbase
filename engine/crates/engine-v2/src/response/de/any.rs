use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};

use crate::response::{Response, ResponseObject, ResponseStringId, ResponseValue};

pub struct AnyFieldsSeed<'resp> {
    pub(super) response: &'resp mut Response,
}

impl<'de, 'resp> DeserializeSeed<'de> for AnyFieldsSeed<'resp> {
    type Value = Vec<(ResponseStringId, ResponseValue)>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AnyFieldsVistor {
            response: self.response,
        })
    }
}

struct AnyFieldsVistor<'resp> {
    response: &'resp mut Response,
}

impl<'de, 'resp> Visitor<'de> for AnyFieldsVistor<'resp> {
    type Value = Vec<(ResponseStringId, ResponseValue)>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an object")
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut fields = vec![];

        while let Some(key) = visitor.next_key::<&str>()? {
            let field_name = self.response.fields.intern_field_name(key);
            let value = visitor.next_value_seed(AnyNodeSeed {
                response: self.response,
            })?;
            fields.push((field_name, value));
        }

        Ok(fields)
    }
}

struct AnyNodeSeed<'resp> {
    response: &'resp mut Response,
}

impl<'de, 'resp> DeserializeSeed<'de> for AnyNodeSeed<'resp> {
    type Value = ResponseValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AnyNodeVisitor {
            response: self.response,
        })
    }
}

struct AnyNodeVisitor<'resp> {
    response: &'resp mut Response,
}

impl<'de, 'resp> Visitor<'de> for AnyNodeVisitor<'resp> {
    type Value = ResponseValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a node")
    }

    #[inline]
    fn visit_bool<E>(self, value: bool) -> Result<ResponseValue, E> {
        Ok(ResponseValue::Bool(value))
    }

    #[inline]
    fn visit_i64<E>(self, value: i64) -> Result<ResponseValue, E> {
        Ok(ResponseValue::Number(value.into()))
    }

    #[inline]
    fn visit_u64<E>(self, value: u64) -> Result<ResponseValue, E> {
        Ok(ResponseValue::Number(value.into()))
    }

    #[inline]
    fn visit_f64<E>(self, value: f64) -> Result<ResponseValue, E> {
        Ok(serde_json::Number::from_f64(value)
            .map(ResponseValue::Number)
            .unwrap_or(ResponseValue::Null))
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<ResponseValue, E> {
        Ok(ResponseValue::String(value.to_string()))
    }

    #[inline]
    fn visit_string<E>(self, value: String) -> Result<ResponseValue, E> {
        Ok(ResponseValue::String(value))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<ResponseValue, E> {
        Ok(ResponseValue::Null)
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<ResponseValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        AnyNodeSeed {
            response: self.response,
        }
        .deserialize(deserializer)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<ResponseValue, E> {
        Ok(ResponseValue::Null)
    }

    #[inline]
    fn visit_seq<V>(self, mut visitor: V) -> Result<ResponseValue, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Some(elem) = visitor.next_element_seed(AnyNodeSeed {
            response: self.response,
        })? {
            vec.push(elem);
        }

        Ok(ResponseValue::List(vec))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<ResponseValue, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut fields = vec![];

        while let Some(key) = visitor.next_key::<&str>()? {
            let field_name = self.response.fields.intern_field_name(key);
            let value = visitor.next_value_seed(AnyNodeSeed {
                response: self.response,
            })?;
            fields.push((field_name, value));
        }

        let object_node_id = self.response.push_object(ResponseObject {
            object_id: None,
            fields,
        });
        Ok(ResponseValue::Object(object_node_id))
    }
}
