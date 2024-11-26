use schema::{EntityDefinition, FieldSetRecord, ResolverDefinition};
use walker::Walk;

use crate::{
    operation::{QueryPartition, ResponseObjectSetDefinitionId},
    resolver::Resolver,
    response::{ConcreteShape, ConcreteShapeId},
};

use super::{Plan, PlanSelectionSet};

impl<'a> Plan<'a> {
    // Not providing too easy access to the query partition as it exposes the unfiltered fields
    // before query modifications. It's likely not what you want.
    fn query_partition(&self) -> QueryPartition<'a> {
        self.as_ref().query_partition_id.walk(self.ctx)
    }

    pub(crate) fn input_id(&self) -> ResponseObjectSetDefinitionId {
        self.query_partition().input_id
    }
    pub(crate) fn entity_definition(&self) -> EntityDefinition<'a> {
        self.query_partition().entity_definition()
    }
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.query_partition().resolver_definition()
    }
    pub(crate) fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet {
            ctx: self.ctx,
            item: self.query_partition().selection_set_record,
            requires_typename: false,
        }
    }
    pub(crate) fn shape_id(&self) -> ConcreteShapeId {
        self.query_partition().shape_id
    }
    pub(crate) fn shape(&self) -> ConcreteShape<'a> {
        self.query_partition().shape_id.walk(self.ctx)
    }
}
