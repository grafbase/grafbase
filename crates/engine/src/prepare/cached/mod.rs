mod builder;
mod document;
mod error;
mod extension;
mod query_plan;
mod shape;

use grafbase_telemetry::graphql::OperationType;
use id_newtypes::IdRange;
use operation::{Operation, OperationContext};
use schema::Schema;
use walker::{Iter, Walk};

pub(crate) use document::*;
pub(crate) use error::*;
pub(crate) use extension::*;
pub(crate) use query_plan::*;
pub(crate) use shape::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CachedOperation {
    pub(crate) document: OperationDocument<'static>,
    pub(crate) operation: Operation,
    pub(crate) query_plan: QueryPlan,
    pub(crate) shapes: Shapes,
}

/// Solving is divided in roughly three steps:
/// 1. Run the query_solver crate to generate the SolutionGraph, defining what resolver to use for
///    which part of query and all the field dependencies.
/// 2. Take the SolutionGraph and the BoundOperation to create all the QueryPartitions in the SolvedOperation
/// 3. Compute all the field shapes for each partition.
pub(crate) fn solve(
    schema: &Schema,
    document: OperationDocument<'_>,
    operation: Operation,
) -> SolveResult<CachedOperation> {
    builder::Solver::build(schema, document, operation)?.solve()
}

#[derive(Clone, Copy)]
pub(crate) struct CachedOperationContext<'a> {
    pub schema: &'a Schema,
    pub cached: &'a CachedOperation,
}

impl<'a> From<CachedOperationContext<'a>> for &'a Schema {
    fn from(ctx: CachedOperationContext<'a>) -> Self {
        ctx.schema
    }
}

impl<'a> From<CachedOperationContext<'a>> for OperationContext<'a> {
    fn from(ctx: CachedOperationContext<'a>) -> Self {
        OperationContext {
            schema: ctx.schema,
            operation: &ctx.cached.operation,
        }
    }
}

impl<'a> CachedOperationContext<'a> {
    pub(in crate::prepare) fn query_partitions(&self) -> impl Iter<Item = QueryPartition<'a>> + 'a {
        IdRange::<QueryPartitionId>::from(0..self.cached.query_plan.partitions.len()).walk(*self)
    }

    pub(in crate::prepare) fn data_fields(&self) -> impl Iter<Item = PartitionDataField<'a>> + 'a {
        IdRange::<PartitionDataFieldId>::from(0..self.cached.query_plan.data_fields.len()).walk(*self)
    }

    pub(in crate::prepare) fn query_modifiers(&self) -> impl Iter<Item = QueryModifier<'a>> + 'a {
        self.cached.query_plan.query_modifiers.walk(*self)
    }

    pub(in crate::prepare) fn response_modifier_definitions(
        &self,
    ) -> impl Iter<Item = ResponseModifierDefinition<'a>> + 'a {
        self.cached.query_plan.response_modifier_definitions.walk(*self)
    }

    // pub(in crate::operation) fn response_modifier_rules(
    //     &self,
    // ) -> impl Iter<Item = (ResponseModifierRule, impl Iterator<Item = DataField<'a>> + 'a)> + 'a {
    //     let ctx = *self;
    //     self.operation
    //         .response_modifier_rule_to_impacted_fields
    //         .iter()
    //         .map(move |item| (item.rule, item.impacted_field_ids.walk(ctx)))
    // }
}

impl CachedOperation {
    pub(crate) fn ty(&self) -> OperationType {
        self.operation.attributes.ty
    }
}
