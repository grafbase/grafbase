use schema::{EntityDefinition, ResolverDefinition};
use walker::Walk;

use crate::prepare::{ConcreteShape, ConcreteShapeId, QueryPartition, ResponseObjectSetDefinitionId};

use super::{Plan, SubgraphSelectionSet};

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
    #[allow(unused)]
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.query_partition().resolver_definition()
    }
    pub(crate) fn selection_set(&self) -> SubgraphSelectionSet<'a> {
        self.ctx.view(self.query_partition_id).selection_set()
    }
    pub(crate) fn shape_id(&self) -> ConcreteShapeId {
        self.query_partition().shape_id
    }
    pub(crate) fn shape(&self) -> ConcreteShape<'a> {
        self.query_partition().shape_id.walk(self.ctx)
    }
}
