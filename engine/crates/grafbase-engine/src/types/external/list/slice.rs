use std::{borrow::Cow, sync::Arc};

use graph_entities::ResponseNodeId;

use crate::{
    parser::types::Field, registry, resolver_utils::resolve_list_native, ContextSelectionSet, InputValueError,
    InputValueResult, LegacyInputType, LegacyOutputType, Positioned, ServerResult, Value,
};

#[async_trait::async_trait]
impl<'a, T: LegacyOutputType + 'a> LegacyOutputType for &'a [T] {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> crate::registry::MetaFieldType {
        format!("[{}]!", T::qualified_type_name()).into()
    }

    fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }

    async fn resolve(&self, ctx: &ContextSelectionSet<'_>, field: &Positioned<Field>) -> ServerResult<ResponseNodeId> {
        resolve_list_native(ctx, field, self.iter(), Some(self.len())).await
    }
}

macro_rules! impl_output_slice_for_smart_ptr {
    ($ty:ty) => {
        #[async_trait::async_trait]
        impl<T: LegacyOutputType> LegacyOutputType for $ty {
            fn type_name() -> Cow<'static, str> {
                Cow::Owned(format!("[{}]", T::qualified_type_name()))
            }

            fn qualified_type_name() -> crate::registry::MetaFieldType {
                format!("[{}]!", T::qualified_type_name()).into()
            }

            fn create_type_info(registry: &mut registry::Registry) -> crate::registry::MetaFieldType {
                T::create_type_info(registry);
                Self::qualified_type_name()
            }

            async fn resolve(
                &self,
                ctx: &ContextSelectionSet<'_>,
                field: &Positioned<Field>,
            ) -> ServerResult<ResponseNodeId> {
                resolve_list_native(ctx, field, self.iter(), Some(self.len())).await
            }
        }
    };
}

impl_output_slice_for_smart_ptr!(Box<[T]>);
impl_output_slice_for_smart_ptr!(Arc<[T]>);

macro_rules! impl_input_slice_for_smart_ptr {
    ($ty:ty) => {
        impl<T: LegacyInputType> LegacyInputType for $ty {
            type RawValueType = Self;

            fn type_name() -> Cow<'static, str> {
                Cow::Owned(format!("[{}]", T::qualified_type_name()))
            }

            fn qualified_type_name() -> crate::registry::InputValueType {
                format!("[{}]!", T::qualified_type_name()).into()
            }

            fn create_type_info(registry: &mut registry::Registry) -> crate::registry::InputValueType {
                T::create_type_info(registry);
                Self::qualified_type_name()
            }

            fn parse(value: Option<Value>) -> InputValueResult<Self> {
                match value.unwrap_or_default() {
                    Value::List(values) => values
                        .into_iter()
                        .map(|value| LegacyInputType::parse(Some(value)))
                        .collect::<Result<_, _>>()
                        .map_err(InputValueError::propagate),
                    value => Ok(vec![LegacyInputType::parse(Some(value)).map_err(InputValueError::propagate)?].into()),
                }
            }

            fn to_value(&self) -> Value {
                Value::List(self.iter().map(LegacyInputType::to_value).collect())
            }

            fn as_raw_value(&self) -> Option<&Self::RawValueType> {
                Some(self)
            }
        }
    };
}

impl_input_slice_for_smart_ptr!(Box<[T]>);
impl_input_slice_for_smart_ptr!(Arc<[T]>);
