mod alternative;
mod operation_fields;
mod providable_fields;
mod prune;

use petgraph::stable_graph::NodeIndex;
use providable_fields::{CreateProvidableFieldsTask, CreateRequirementTask, UnplannableField};
use schema::{CompositeTypeId, Schema, TypeDefinitionId};
use walker::Walk;

use crate::{
    DeduplicationId, FieldFlags, FieldNode, QueryFieldId, SplitId, deduplication::DeduplicationMap,
    steps::SolutionSpace,
};

use super::*;

#[derive(id_derives::IndexedFields)]
pub(super) struct QuerySolutionSpaceBuilder<'schema, 'op> {
    schema: &'schema Schema,
    operation: &'op Operation,
    query: QuerySolutionSpace<'schema>,
    create_provideable_fields_task_stack: Vec<CreateProvidableFieldsTask>,
    create_requirement_task_stack: Vec<CreateRequirementTask<'schema>>,
    maybe_unplannable_query_fields_stack: Vec<UnplannableField>,
    current_split: SplitId,
    next_split: usize,
}

impl<'schema> QuerySolutionSpace<'schema> {
    pub(super) fn builder<'op>(
        schema: &'schema Schema,
        operation: &'op Operation,
    ) -> QuerySolutionSpaceBuilder<'schema, 'op>
    where
        'schema: 'op,
    {
        let n = operation.data_fields.len() + operation.typename_fields.len();
        let mut graph = petgraph::stable_graph::StableGraph::with_capacity(n * 2, n * 2);
        let root_node_id = graph.add_node(SpaceNode::Root);

        QuerySolutionSpaceBuilder {
            schema,
            operation,
            query: Query {
                step: SolutionSpace {
                    deduplication_map: DeduplicationMap::with_capacity(
                        operation.data_fields.len() + operation.typename_fields.len(),
                    ),
                },
                root_node_id,
                graph,
                fields: Vec::with_capacity(n),
                shared_type_conditions: Vec::new(),
                deduplicated_flat_sorted_executable_directives: Default::default(),
            },
            create_provideable_fields_task_stack: Vec::new(),
            create_requirement_task_stack: Vec::new(),
            maybe_unplannable_query_fields_stack: Vec::new(),
            current_split: SplitId::from(0usize),
            next_split: 1,
        }
    }
}

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn build(mut self) -> crate::Result<QuerySolutionSpace<'schema>> {
        self.ingest_operation_fields()?;

        self.create_providable_fields_tasks_for_subselection(providable_fields::Parent {
            query_field_node_ix: self.query.root_node_id,
            providable_field_or_root_ix: self.query.root_node_id,
            output_type: CompositeTypeId::Object(self.operation.root_object_id),
        });

        loop {
            self.loop_over_tasks();
            if let Some(unplannable_field) = self.maybe_unplannable_query_fields_stack.pop() {
                self.handle_unplannable_field(unplannable_field)?;
                while let Some(unplannable_field) = self.maybe_unplannable_query_fields_stack.pop() {
                    self.handle_unplannable_field(unplannable_field)?;
                }
            } else {
                break;
            }
        }

        tracing::debug!("Query before pruning:\n{}", self.query.to_pretty_dot_graph(self.ctx()));
        self.prune_resolvers_not_leading_any_leafs();

        Ok(self.query)
    }

    fn loop_over_tasks(&mut self) {
        // We first ingest all fields so that requirements can reference them. We use a double
        // stack as requirement may means adding new fields and adding new fields may add new
        // requirements.
        loop {
            while let Some(task) = self.create_provideable_fields_task_stack.pop() {
                self.create_providable_fields(task);
            }
            if let Some(task) = self.create_requirement_task_stack.pop() {
                self.create_requirement(task)
            } else {
                break;
            }
        }
    }

    fn push_query_field_node(&mut self, id: QueryFieldId, flags: FieldFlags) -> NodeIndex {
        let dedup_id = self.query.get_or_insert_field_deduplication_id(self.ctx(), id);
        self.push_query_field_node_with_dedup_id(id, dedup_id, flags)
    }

    fn push_query_field_node_with_dedup_id(
        &mut self,
        id: QueryFieldId,
        dedup_id: DeduplicationId,
        mut flags: FieldFlags,
    ) -> NodeIndex {
        if let Some(field_definition) = self.query[id].definition_id {
            match field_definition.walk(self.schema).ty().definition_id {
                TypeDefinitionId::Scalar(_) | TypeDefinitionId::Enum(_) => {
                    flags |= FieldFlags::LEAF_NODE;
                }
                _ => (),
            }
        }

        let query_field = SpaceNode::Field(FieldNode {
            id,
            dedup_id,
            split_id: self.current_split,
            flags,
        });

        self.query.graph.add_node(query_field)
    }

    fn ctx(&self) -> OperationContext<'op> {
        OperationContext {
            schema: self.schema,
            operation: self.operation,
        }
    }
}
