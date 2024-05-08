use std::{borrow::Cow, collections::HashMap, fmt::Display, hash::Hash, str::FromStr};

use engine_parser::{types::Field, Positioned};
use engine_value::{from_value, to_value};
use graph_entities::ResponseNodeId;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    graph::selection_set_into_node,
    registry::{self, LegacyRegistryExt, MetaType, Registry, ScalarType},
    ContextSelectionSetLegacy, InputValueError, InputValueResult, LegacyInputType, LegacyOutputType, Name,
    ServerResult, Value,
};

impl<K, V> LegacyInputType for HashMap<K, V>
where
    K: ToString + FromStr + Eq + Hash + Send + Sync,
    K::Err: Display,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSONObject")
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::InputValueType {
        registry.create_input_type::<Self, _>(|_| {
            MetaType::Scalar(ScalarType {
                name: <Self as LegacyInputType>::type_name().to_string(),
                description: Some("A scalar that can represent any JSON Object value.".to_string()),
                is_valid: Some(|_| true),
                specified_by_url: None,
                parser: registry::ScalarParser::BestEffort,
            })
        })
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        let value = value.unwrap_or_default();
        match value {
            Value::Object(map) => map
                .into_iter()
                .map(|(name, value)| {
                    Ok((
                        K::from_str(&name)
                            .map_err(|err| InputValueError::<Self>::custom(format!("object key: {err}")))?,
                        from_value(value).map_err(|err| format!("object value: {err}"))?,
                    ))
                })
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        let mut map = IndexMap::new();
        for (name, value) in self {
            map.insert(Name::new(name.to_string()), to_value(value).unwrap_or_default());
        }
        Value::Object(map)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

#[async_trait::async_trait]
impl<K, V> LegacyOutputType for HashMap<K, V>
where
    K: ToString + Eq + Hash + Send + Sync,
    V: Serialize + Send + Sync,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSONObject")
    }

    fn create_type_info(registry: &mut Registry) -> crate::registry::MetaFieldType {
        registry.create_output_type::<Self, _>(|_| {
            MetaType::Scalar(ScalarType {
                name: <Self as LegacyOutputType>::type_name().to_string(),
                description: Some("A scalar that can represent any JSON Object value.".to_string()),
                is_valid: Some(|_| true),
                specified_by_url: None,
                parser: registry::ScalarParser::BestEffort,
            })
        })
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        let mut map = IndexMap::new();
        for (name, value) in self {
            map.insert(Name::new(name.to_string()), to_value(value).unwrap_or_default());
        }

        Ok(selection_set_into_node(Value::Object(map), ctx).await)
    }
}
