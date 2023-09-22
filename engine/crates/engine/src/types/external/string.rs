use std::borrow::Cow;

use graph_entities::{CompactValue, ResponseNodeId};

use crate::{
    parser::types::Field, registry, registry::Registry, ContextExt, ContextSelectionSetLegacy, InputValueError,
    InputValueResult, LegacyInputType, LegacyOutputType, LegacyScalarType, Positioned, Scalar, ServerResult, Value,
};

/// The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.
#[Scalar(internal)]
impl LegacyScalarType for String {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

macro_rules! impl_input_string_for_smart_ptr {
    ($ty:ty) => {
        #[async_trait::async_trait]
        impl LegacyInputType for $ty {
            type RawValueType = Self;

            fn type_name() -> Cow<'static, str> {
                Cow::Borrowed("String")
            }

            fn create_type_info(registry: &mut Registry) -> crate::registry::InputValueType {
                <String as LegacyInputType>::create_type_info(registry)
            }

            fn parse(value: Option<Value>) -> InputValueResult<Self> {
                let value = value.unwrap_or_default();
                match value {
                    Value::String(s) => Ok(s.into()),
                    _ => Err(InputValueError::expected_type(value)),
                }
            }

            fn to_value(&self) -> Value {
                Value::String(self.to_string())
            }

            fn as_raw_value(&self) -> Option<&Self::RawValueType> {
                Some(self)
            }
        }
    };
}

impl_input_string_for_smart_ptr!(Box<str>);
impl_input_string_for_smart_ptr!(std::sync::Arc<str>);

#[async_trait::async_trait]
impl LegacyOutputType for str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType {
        <String as LegacyOutputType>::create_type_info(registry)
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSetLegacy<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<ResponseNodeId> {
        let mut graph = ctx.response().await;
        Ok(graph.insert_node(CompactValue::String(self.to_string())))
    }
}
