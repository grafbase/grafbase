use operation::{
    InputValueContext, QueryInputValueRecord, QueryOrSchemaInputValue, QueryOrSchemaInputValueId,
    QueryOrSchemaInputValueView, Variables,
};
use query_solver::QueryOrSchemaFieldArgumentIds;
use schema::{
    InputValueDefinition, InputValueDefinitionId, InputValueSerdeError, InputValueSet, SchemaInputValueRecord,
};
use serde::{
    de::{value::MapDeserializer, IntoDeserializer, Visitor},
    forward_to_deserialize_any,
};
use walker::Walk;

use crate::prepare::CachedOperationContext;

#[derive(Clone, Copy)]
pub(crate) struct PartitionFieldArguments<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    ids: QueryOrSchemaFieldArgumentIds,
}

impl<'ctx> PartitionFieldArguments<'ctx> {
    pub(crate) fn len(&self) -> usize {
        match self.ids {
            QueryOrSchemaFieldArgumentIds::Query(ids) => ids.len(),
            QueryOrSchemaFieldArgumentIds::Schema(ids) => ids.len(),
        }
    }

    pub(crate) fn view<'v, 's, 'view>(
        &self,
        selection_set: &'s InputValueSet,
        variables: &'v Variables,
    ) -> PartitionFieldArgumentsView<'view>
    where
        'ctx: 'view,
        'v: 'view,
        's: 'view,
    {
        PartitionFieldArgumentsView {
            ctx: self.ctx,
            variables,
            ids: self.ids,
            selection_set,
        }
    }

    #[track_caller]
    pub(crate) fn get_arg_value_as<'v, 't, T: serde::Deserialize<'t>>(&self, name: &str, variables: &'v Variables) -> T
    where
        'v: 't,
        'ctx: 't,
    {
        T::deserialize(
            self.get_arg_value_opt(name, variables)
                .expect("Argument is not nullable"),
        )
        .expect("Invalid argument type.")
    }

    pub(crate) fn get_arg_value_opt<'t, 'v>(
        &self,
        name: &str,
        variables: &'v Variables,
    ) -> Option<QueryOrSchemaInputValue<'t>>
    where
        'v: 't,
        'ctx: 't,
    {
        let ctx = InputValueContext {
            schema: self.ctx.schema,
            query_input_values: &self.ctx.cached.operation.query_input_values,
            variables,
        };
        match self.ids {
            QueryOrSchemaFieldArgumentIds::Query(ids) => ids.walk(self.ctx).find_map(|arg| {
                if arg.definition().name() == name {
                    let value = arg.value_id.walk(ctx);
                    if value.is_undefined() {
                        arg.definition().default_value().map(QueryOrSchemaInputValue::Schema)
                    } else {
                        Some(QueryOrSchemaInputValue::Query(value))
                    }
                } else {
                    None
                }
            }),
            QueryOrSchemaFieldArgumentIds::Schema(ids) => ids.walk(self.ctx).find_map(|arg| {
                if arg.definition().name() == name {
                    Some(QueryOrSchemaInputValue::Schema(arg.value()))
                } else {
                    None
                }
            }),
        }
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for QueryOrSchemaFieldArgumentIds {
    type Walker<'w>
        = PartitionFieldArguments<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        PartitionFieldArguments {
            ctx: ctx.into(),
            ids: self,
        }
    }
}

impl std::fmt::Debug for PartitionFieldArguments<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.into_iter()
                    .map(|arg| (arg.definition().name(), arg.value_as_sanitized_query_const_value_str())),
            )
            .finish()
    }
}

impl<'a> IntoIterator for PartitionFieldArguments<'a> {
    type Item = PartitionFieldArgument<'a>;
    type IntoIter = PartitionFieldArgumentsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PartitionFieldArgumentsIterator {
            ctx: self.ctx,
            args: match self.ids {
                QueryOrSchemaFieldArgumentIds::Query(ids) => self.ctx.cached.operation[ids]
                    .iter()
                    .map(|arg| (arg.definition_id, arg.value_id.into()))
                    .collect(),
                QueryOrSchemaFieldArgumentIds::Schema(ids) => self.ctx.schema[ids]
                    .iter()
                    .map(|arg| (arg.definition_id, arg.value_id.into()))
                    .collect(),
            },
        }
    }
}

pub(crate) struct PartitionFieldArgumentsIterator<'a> {
    ctx: CachedOperationContext<'a>,
    args: Vec<(InputValueDefinitionId, QueryOrSchemaInputValueId)>,
}

impl<'a> Iterator for PartitionFieldArgumentsIterator<'a> {
    type Item = PartitionFieldArgument<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.args.pop().map(|(definition_id, value_id)| PartitionFieldArgument {
            ctx: self.ctx,
            definition_id,
            value_id,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PartitionFieldArgument<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) definition_id: InputValueDefinitionId,
    pub(crate) value_id: QueryOrSchemaInputValueId,
}

impl<'a> PartitionFieldArgument<'a> {
    pub(crate) fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.ctx)
    }

    /// Used for GraphQL query generation to only include values in the query string that would be
    /// present after query sanitization.
    pub(crate) fn value_as_sanitized_query_const_value_str(&self) -> Option<&'a str> {
        match self.value_id {
            QueryOrSchemaInputValueId::Query(id) => Some(match &self.ctx.cached.operation.query_input_values[id] {
                QueryInputValueRecord::EnumValue(id) => self.ctx.schema.walk(*id).name(),
                QueryInputValueRecord::Boolean(b) => {
                    if *b {
                        "true"
                    } else {
                        "false"
                    }
                }
                QueryInputValueRecord::DefaultValue(id) => match &self.ctx.schema[*id] {
                    SchemaInputValueRecord::EnumValue(id) => self.ctx.schema.walk(*id).name(),
                    SchemaInputValueRecord::Boolean(b) => {
                        if *b {
                            "true"
                        } else {
                            "false"
                        }
                    }
                    _ => return None,
                },
                _ => return None,
            }),
            QueryOrSchemaInputValueId::Schema(id) => Some(match &self.ctx.schema[id] {
                SchemaInputValueRecord::EnumValue(id) => self.ctx.schema.walk(*id).name(),
                SchemaInputValueRecord::Boolean(b) => {
                    if *b {
                        "true"
                    } else {
                        "false"
                    }
                }
                _ => return None,
            }),
        }
    }

    #[allow(unused)]
    pub(crate) fn value<'v, 'w>(&self, variables: &'v Variables) -> QueryOrSchemaInputValue<'w>
    where
        'v: 'w,
        'a: 'w,
    {
        self.value_id.walk(InputValueContext {
            schema: self.ctx.schema,
            query_input_values: &self.ctx.cached.operation.query_input_values,
            variables,
        })
    }
}

pub(crate) struct PartitionFieldArgumentsView<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) variables: &'a Variables,
    ids: QueryOrSchemaFieldArgumentIds,
    selection_set: &'a InputValueSet,
}

impl serde::Serialize for PartitionFieldArgumentsView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.ids {
            QueryOrSchemaFieldArgumentIds::Query(ids) => {
                let ctx = InputValueContext {
                    schema: self.ctx.schema,
                    query_input_values: &self.ctx.cached.operation.query_input_values,
                    variables: self.variables,
                };
                serializer.collect_map(ids.walk(self.ctx).filter_map(|arg| {
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
            QueryOrSchemaFieldArgumentIds::Schema(ids) => {
                serializer.collect_map(ids.walk(self.ctx).filter_map(|arg| {
                    self.selection_set
                        .iter()
                        .find(|item| item.id == arg.definition_id)
                        .map(|item| {
                            (
                                arg.definition().name(),
                                QueryOrSchemaInputValueView::Schema(arg.value().with_selection_set(&item.subselection)),
                            )
                        })
                }))
            }
        }
    }
}

impl<'de> serde::Deserializer<'de> for PartitionFieldArgumentsView<'de> {
    type Error = InputValueSerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.ids {
            QueryOrSchemaFieldArgumentIds::Query(ids) => {
                let ctx = InputValueContext {
                    schema: self.ctx.schema,
                    query_input_values: &self.ctx.cached.operation.query_input_values,
                    variables: self.variables,
                };
                MapDeserializer::new(ids.walk(self.ctx).filter_map(|arg| {
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
            QueryOrSchemaFieldArgumentIds::Schema(ids) => MapDeserializer::new(ids.walk(self.ctx).filter_map(|arg| {
                self.selection_set
                    .iter()
                    .find(|item| item.id == arg.definition_id)
                    .map(|item| {
                        (
                            arg.definition().name(),
                            QueryOrSchemaInputValueView::Schema(arg.value().with_selection_set(&item.subselection)),
                        )
                    })
            }))
            .deserialize_any(visitor),
        }
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

impl<'de> IntoDeserializer<'de, InputValueSerdeError> for PartitionFieldArgumentsView<'de> {
    type Deserializer = Self;
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
