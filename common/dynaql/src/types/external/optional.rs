use std::borrow::Cow;

use graph_entities::{CompactValue, ResponseNodeId};

use crate::parser::types::Field;
use crate::{
    registry, ContextSelectionSet, InputValueError, InputValueResult, LegacyInputType, LegacyOutputType, Positioned,
    ServerResult, Value,
};

impl<T: LegacyInputType> LegacyInputType for Option<T> {
    type RawValueType = T::RawValueType;

    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> crate::registry::InputValueType {
        T::type_name().as_ref().into()
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::InputValueType {
        T::create_type_info(registry);
        T::type_name().as_ref().into()
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::Null => Ok(None),
            value => Ok(Some(T::parse(Some(value)).map_err(InputValueError::propagate)?)),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Some(value) => value.to_value(),
            None => Value::Null,
        }
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        match self {
            Some(value) => value.as_raw_value(),
            None => None,
        }
    }
}

#[async_trait::async_trait]
impl<T: LegacyOutputType + Sync> LegacyOutputType for Option<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> crate::registry::MetaFieldType {
        T::type_name().as_ref().into()
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType {
        T::create_type_info(registry);
        T::type_name().as_ref().into()
    }

    async fn resolve(&self, ctx: &ContextSelectionSet<'_>, field: &Positioned<Field>) -> ServerResult<ResponseNodeId> {
        if let Some(inner) = self {
            match LegacyOutputType::resolve(inner, ctx, field).await {
                Ok(value) => Ok(value),
                Err(err) => {
                    ctx.add_error(err);
                    let mut graph = ctx.response_graph.write().await;
                    Ok(graph.insert_node(CompactValue::Null))
                }
            }
        } else {
            let mut graph = ctx.response_graph.write().await;
            Ok(graph.insert_node(CompactValue::Null))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::LegacyInputType;

    #[test]
    fn test_optional_type() {
        assert_eq!(Option::<i32>::type_name(), "Int");
        assert_eq!(Option::<i32>::qualified_type_name().as_str(), "Int");
        assert_eq!(&Option::<i32>::type_name(), "Int");
        assert_eq!(Option::<i32>::qualified_type_name().as_str(), "Int");
    }
}
