use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

use dynaql_parser::types::Field;
use dynaql_parser::Positioned;
use dynaql_value::{from_value, to_value};
use graph_entities::ResponseNodeId;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::graph::selection_set_into_node;
use crate::registry::{self, MetaType, Registry};
use crate::{
    ContextSelectionSet, InputType, InputValueError, InputValueResult, Name, OutputType,
    ServerResult, Value,
};

impl<K, V> InputType for HashMap<K, V>
where
    K: ToString + FromStr + Eq + Hash + Send + Sync,
    K::Err: Display,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSONObject")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_input_type::<Self, _>(|_| MetaType::Scalar {
            name: <Self as InputType>::type_name().to_string(),
            description: Some("A scalar that can represent any JSON Object value.".to_string()),
            is_valid: Some(|_| true),
            visible: None,
            specified_by_url: None,
            parser: registry::ScalarParser::BestEffort,
        })
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        let value = value.unwrap_or_default();
        match value {
            Value::Object(map) => map
                .into_iter()
                .map(|(name, value)| {
                    Ok((
                        K::from_str(&name).map_err(|err| {
                            InputValueError::<Self>::custom(format!("object key: {err}"))
                        })?,
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
            map.insert(
                Name::new(name.to_string()),
                to_value(value).unwrap_or_default(),
            );
        }
        Value::Object(map)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

#[async_trait::async_trait]
impl<K, V> OutputType for HashMap<K, V>
where
    K: ToString + Eq + Hash + Send + Sync,
    V: Serialize + Send + Sync,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("JSONObject")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_output_type::<Self, _>(|_| MetaType::Scalar {
            name: <Self as OutputType>::type_name().to_string(),
            description: Some("A scalar that can represent any JSON Object value.".to_string()),
            is_valid: Some(|_| true),
            visible: None,
            specified_by_url: None,
            parser: registry::ScalarParser::BestEffort,
        })
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        let mut map = IndexMap::new();
        for (name, value) in self {
            map.insert(
                Name::new(name.to_string()),
                to_value(value).unwrap_or_default(),
            );
        }
        let ctx_field = ctx.with_field(field, None, Some(&ctx.item.node));
        let ty = ctx_field
            .schema_env
            .registry
            .types
            .get(Self::type_name().as_ref())
            .expect("If this type is used it should be in the registry");

        Ok(selection_set_into_node(Value::Object(map), ctx, ty).await)
    }
}
