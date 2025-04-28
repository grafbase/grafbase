use operation::{InputValueContext, QueryOrSchemaInputValue, Variables};
use schema::InputValueSet;
use serde::ser::SerializeMap as _;
use walker::Walk as _;

use crate::{
    prepare::{CachedOperationContext, PartitionFieldArgument, PartitionFieldArgumentRecord},
    response::{ResponseObjectView, ResponseObjectsView},
};

use super::PlanValueRecord;

#[derive(Clone, Copy)]
pub(crate) struct PlanFieldArguments<'a> {
    pub ctx: CachedOperationContext<'a>,
    pub records: &'a [PartitionFieldArgumentRecord],
}

impl std::fmt::Debug for PlanFieldArguments<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.into_iter()
                    .map(|arg| (arg.definition().name(), arg.value_as_sanitized_query_const_value_str())),
            )
            .finish()
    }
}

impl<'ctx> PlanFieldArguments<'ctx> {
    pub(crate) fn empty(ctx: CachedOperationContext<'ctx>) -> Self {
        PlanFieldArguments { ctx, records: &[] }
    }

    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }

    pub(crate) fn query_view<'v, 's, 'view>(
        &self,
        selection_set: &'s InputValueSet,
        variables: &'v Variables,
    ) -> PlanFieldArgumentsQueryView<'view>
    where
        'ctx: 'view,
        'v: 'view,
        's: 'view,
    {
        PlanFieldArgumentsQueryView {
            ctx: self.ctx,
            variables,
            arguments: self.records,
            selection_set,
        }
    }

    pub(crate) fn batch_view<'v, 'r, 'view>(
        &self,
        variables: &'v Variables,
        parent_objects: ResponseObjectsView<'r>,
    ) -> PlanFieldArgumentsBatchView<'view>
    where
        'ctx: 'view,
        'v: 'view,
        'r: 'view,
    {
        PlanFieldArgumentsBatchView {
            ctx: self.ctx,
            variables,
            parent_objects,
            arguments: self.records,
        }
    }

    #[allow(unused)]
    pub(crate) fn view<'v, 'r, 'view>(
        &self,
        variables: &'v Variables,
        parent_object: ResponseObjectView<'r>,
    ) -> PlanFieldArgumentsView<'view>
    where
        'ctx: 'view,
        'v: 'view,
        'r: 'view,
    {
        PlanFieldArgumentsView {
            ctx: self.ctx,
            variables,
            parent_object,
            arguments: self.records,
        }
    }

    #[track_caller]
    pub(crate) fn get_arg_value_as<'v, 't, T: serde::Deserialize<'t>>(&self, name: &str, variables: &'v Variables) -> T
    where
        'v: 't,
        'ctx: 't,
    {
        let value = self.records.walk(self.ctx).find_map(|arg| {
            if arg.definition().name() != name {
                return None;
            }
            arg.value(variables)
        });
        T::deserialize(value.expect("Argument is not nullable")).expect("Invalid argument type.")
    }
}

impl<'a> IntoIterator for PlanFieldArguments<'a> {
    type Item = PartitionFieldArgument<'a>;
    type IntoIter = PlanFieldArgumentsIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PlanFieldArgumentsIterator {
            ctx: self.ctx,
            inner: self.records.iter(),
        }
    }
}

pub(crate) struct PlanFieldArgumentsIterator<'a> {
    ctx: CachedOperationContext<'a>,
    inner: std::slice::Iter<'a, PartitionFieldArgumentRecord>,
}

impl<'a> Iterator for PlanFieldArgumentsIterator<'a> {
    type Item = PartitionFieldArgument<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|arg| arg.walk(self.ctx))
    }
}

pub(crate) struct PlanFieldArgumentsQueryView<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) variables: &'a Variables,
    arguments: &'a [PartitionFieldArgumentRecord],
    selection_set: &'a InputValueSet,
}

impl serde::Serialize for PlanFieldArgumentsQueryView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ctx = InputValueContext {
            schema: self.ctx.schema,
            query_input_values: &self.ctx.cached.operation.query_input_values,
            variables: self.variables,
        };
        if let InputValueSet::SelectionSet(selection_set) = self.selection_set {
            serializer.collect_map(self.arguments.walk(self.ctx).filter_map(|arg| {
                selection_set
                    .iter()
                    .find(|item| item.definition_id == arg.definition_id)
                    .map(|item| {
                        let value = arg
                            .value_record
                            .as_schema_or_query_input_value()
                            .expect("TODO GB-8938")
                            .walk(ctx)
                            .with_selection_set(&item.subselection);
                        (arg.definition().name(), value)
                    })
            }))
        } else {
            serializer.collect_map(self.arguments.walk(self.ctx).filter_map(|arg| {
                let value = arg.value_record.as_schema_or_query_input_value().unwrap().walk(ctx);
                if value.is_undefined() {
                    arg.definition()
                        .default_value()
                        .map(|value| (arg.definition().name(), QueryOrSchemaInputValue::Schema(value)))
                } else {
                    Some((arg.definition().name(), value))
                }
            }))
        }
    }
}

pub(crate) struct PlanFieldArgumentsBatchView<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) variables: &'a Variables,
    parent_objects: ResponseObjectsView<'a>,
    arguments: &'a [PartitionFieldArgumentRecord],
}

impl serde::Serialize for PlanFieldArgumentsBatchView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ctx = InputValueContext {
            schema: self.ctx.schema,
            query_input_values: &self.ctx.cached.operation.query_input_values,
            variables: self.variables,
        };
        let mut map = serializer.serialize_map(Some(self.arguments.len()))?;
        for arg in self.arguments.walk(self.ctx) {
            match arg.value_record {
                PlanValueRecord::Value(id) => {
                    let value = id.walk(ctx);
                    if value.is_undefined() {
                        if let Some(value) = arg.definition().default_value() {
                            map.serialize_entry(arg.definition().name(), &value)?;
                        }
                    } else {
                        map.serialize_entry(arg.definition().name(), &value)?;
                    }
                }
                PlanValueRecord::Injection(injection) => {
                    map.serialize_entry(
                        arg.definition().name(),
                        &self.parent_objects.clone().for_injection(injection),
                    )?;
                }
            }
        }
        map.end()
    }
}

pub(crate) struct PlanFieldArgumentsView<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(in crate::prepare::cached::query_plan) variables: &'a Variables,
    parent_object: ResponseObjectView<'a>,
    arguments: &'a [PartitionFieldArgumentRecord],
}

impl serde::Serialize for PlanFieldArgumentsView<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ctx = InputValueContext {
            schema: self.ctx.schema,
            query_input_values: &self.ctx.cached.operation.query_input_values,
            variables: self.variables,
        };
        let mut map = serializer.serialize_map(Some(self.arguments.len()))?;
        for arg in self.arguments.walk(self.ctx) {
            match arg.value_record {
                PlanValueRecord::Value(id) => {
                    let value = id.walk(ctx);
                    if value.is_undefined() {
                        if let Some(value) = arg.definition().default_value() {
                            map.serialize_entry(arg.definition().name(), &value)?;
                        }
                    } else {
                        map.serialize_entry(arg.definition().name(), &value)?;
                    }
                }
                PlanValueRecord::Injection(injection) => {
                    map.serialize_entry(arg.definition().name(), &self.parent_object.for_injection(injection))?;
                }
            }
        }
        map.end()
    }
}
