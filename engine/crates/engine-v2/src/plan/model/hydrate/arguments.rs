use id_newtypes::IdRange;
use schema::{InputValueSerdeError, InputValueSet, Schema};
use serde::{
    de::{value::MapDeserializer, IntoDeserializer, Visitor},
    forward_to_deserialize_any,
};
use walker::Walk;

use crate::operation::{InputValueContext, QueryOrSchemaInputValueView, Variables};

use crate::plan::model::{FieldArgumentId, OperationPlan, PlanContext};

#[derive(Clone, Copy)]
pub struct HydratedFieldArguments<'a> {
    pub(in crate::plan::model) schema: &'a Schema,
    pub(in crate::plan::model) operation_plan: &'a OperationPlan,
    pub(in crate::plan::model) variables: &'a Variables,
    pub(in crate::plan::model) ids: IdRange<FieldArgumentId>,
}

impl<'a> HydratedFieldArguments<'a> {
    pub fn with_selection_set<'w, 'i>(self, selection_set: &'i InputValueSet) -> HydratedFieldArgumentsView<'w>
    where
        'i: 'w,
        'a: 'w,
    {
        HydratedFieldArgumentsView {
            arguments: self,
            selection_set,
        }
    }
}

pub struct HydratedFieldArgumentsView<'a> {
    pub(super) arguments: HydratedFieldArguments<'a>,
    pub(super) selection_set: &'a InputValueSet,
}

impl<'a> serde::Serialize for HydratedFieldArgumentsView<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let HydratedFieldArguments {
            schema,
            operation_plan,
            variables,
            ids,
        } = self.arguments;
        let ctx = InputValueContext {
            schema,
            query_input_values: &operation_plan.query_input_values,
            variables,
        };
        let plan_ctx = PlanContext { schema, operation_plan };
        serializer.collect_map(ids.walk(plan_ctx).filter_map(|arg| {
            if let Some(item) = self.selection_set.iter().find(|item| item.id == arg.definition_id) {
                let value = arg.value_id.walk(ctx);
                if value.is_undefined() {
                    arg.definition().default_value().map(|value| {
                        (
                            arg.definition().name(),
                            QueryOrSchemaInputValueView::Schema(value.with_selection_set(&item.subselection)),
                        )
                    })
                } else {
                    Some((
                        arg.definition().name(),
                        QueryOrSchemaInputValueView::Query(value.with_selection_set(&item.subselection)),
                    ))
                }
            } else {
                None
            }
        }))
    }
}

impl<'de> serde::Deserializer<'de> for HydratedFieldArgumentsView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let HydratedFieldArguments {
            schema,
            operation_plan,
            variables,
            ids,
        } = self.arguments;
        let ctx = InputValueContext {
            schema,
            query_input_values: &operation_plan.query_input_values,
            variables,
        };
        let plan_ctx = PlanContext { schema, operation_plan };
        MapDeserializer::new(ids.walk(plan_ctx).filter_map(|arg| {
            if let Some(item) = self.selection_set.iter().find(|item| item.id == arg.definition_id) {
                let value = arg.value_id.walk(ctx);
                if value.is_undefined() {
                    arg.definition().default_value().map(|value| {
                        (
                            arg.definition().name(),
                            QueryOrSchemaInputValueView::Schema(value.with_selection_set(&item.subselection)),
                        )
                    })
                } else {
                    Some((
                        arg.definition().name(),
                        QueryOrSchemaInputValueView::Query(value.with_selection_set(&item.subselection)),
                    ))
                }
            } else {
                None
            }
        }))
        .deserialize_any(visitor)
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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
}

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for HydratedFieldArgumentsView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
