use std::{
    collections::{HashMap, VecDeque},
    hash::BuildHasherDefault,
};

use fxhash::FxHasher32;
use operation::{ExecutableDirectiveId, OperationContext, QueryPosition};
use petgraph::Direction;
use schema::{CompositeTypeId, TypeSystemDirective};
use walker::Walk;

use crate::{
    are_arguments_equivalent, DeduplicatedFlatExecutableDirectivesId, FieldFlags, QueryField, QuerySelectionSet,
    QuerySelectionSetId, QueryTypenameField,
};

use super::{builder::QuerySolutionSpaceBuilder, providable_fields::CreateRequirementTask, SpaceEdge, SpaceNode};

struct IngestSelectionSet<'op> {
    id: QuerySelectionSetId,
    selection_set: operation::SelectionSet<'op>,
}

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn ingest_operation_fields(&mut self) -> crate::Result<()> {
        self.query.selection_sets.push(QuerySelectionSet {
            parent_node_ix: self.query.root_node_ix,
            output_type_id: self.operation.root_object_id.into(),
            typename_node_ix: None,
            typename_fields: Vec::new(),
        });
        let queue = vec![IngestSelectionSet {
            id: QuerySelectionSetId::from(0usize),
            selection_set: OperationContext {
                schema: self.schema,
                operation: self.operation,
            }
            .root_selection_set(),
        }]
        .into();
        OperationFieldsIngestor {
            builder: self,
            queue,
            next_query_position: 0,
            parent_type_conditions: Vec::new(),
            parent_directive_ids: Vec::new(),
            response_key_bloom_filter: 0,
        }
        .ingest()?;
        Ok(())
    }
}

#[derive(id_derives::IndexedFields)]
struct OperationFieldsIngestor<'schema, 'op, 'builder> {
    builder: &'builder mut QuerySolutionSpaceBuilder<'schema, 'op>,
    // Needs to be a queue to have the right query_position for fields.
    queue: VecDeque<IngestSelectionSet<'op>>,
    next_query_position: u16,
    // Temporary structures for DFS
    parent_type_conditions: Vec<CompositeTypeId>,
    parent_directive_ids: Vec<ExecutableDirectiveId>,
    response_key_bloom_filter: usize,
}

impl<'schema, 'op, 'builder> OperationFieldsIngestor<'schema, 'op, 'builder>
where
    'schema: 'op,
    'op: 'builder,
{
    fn ingest(&mut self) -> crate::Result<()> {
        let mut selection_set_to_response_key_bloom_filter: HashMap<
            QuerySelectionSetId,
            usize,
            BuildHasherDefault<FxHasher32>,
        > = HashMap::with_capacity_and_hasher(self.builder.operation.data_fields.len() >> 2, Default::default());
        while let Some(IngestSelectionSet { id, selection_set }) = self.queue.pop_front() {
            self.parent_type_conditions.clear();
            self.parent_directive_ids.clear();
            let bloom_filter = selection_set_to_response_key_bloom_filter.entry(id).or_default();
            self.response_key_bloom_filter = *bloom_filter;
            self.rec_ingest_selection_set(id, selection_set)?;
            *bloom_filter = self.response_key_bloom_filter;
        }

        Ok(())
    }

    fn next_query_position(&mut self) -> QueryPosition {
        let p = self.next_query_position;
        self.next_query_position += 1;
        p.into()
    }

    fn rec_ingest_selection_set(
        &mut self,
        id: QuerySelectionSetId,
        selection_set: operation::SelectionSet<'op>,
    ) -> crate::Result<()> {
        for selection in selection_set {
            match selection {
                operation::Selection::Field(field) => {
                    if let operation::Field::Data(field) = field {
                        let ty = field.definition().parent_entity_id.as_composite_type();
                        if !self.can_be_present(id, ty) {
                            // This field can never appear, likely comes from a common
                            // fragment. In the Operation validation we only verify that fragments have
                            // a common element with their direct parent.
                            continue;
                        }
                    }
                    self.add_operation_field(id, field)?;
                }
                operation::Selection::FragmentSpread(spread) => {
                    let fragment = spread.fragment();
                    let ty = fragment.type_condition_id;
                    if !self.can_be_present(id, ty) {
                        // This selection can never appear, likely comes from a common
                        // fragment. In the Operation validation we only verify that fragments have
                        // a common element with their direct
                        continue;
                    }

                    let n = self.parent_directive_ids.len();
                    self.parent_directive_ids.extend_from_slice(&spread.directive_ids);
                    self.parent_type_conditions.push(ty);
                    self.rec_ingest_selection_set(id, fragment.selection_set())?;
                    self.parent_type_conditions.pop();
                    self.parent_directive_ids.truncate(n);
                }
                operation::Selection::InlineFragment(fragment) => {
                    if let Some(ty) = fragment.type_condition_id {
                        if !self.can_be_present(id, ty) {
                            // This selection can never appear, likely comes from a common
                            // fragment. In the Operation validation we only verify that fragments have
                            // a common element with their direct
                            continue;
                        }

                        let n = self.parent_directive_ids.len();
                        self.parent_directive_ids.extend_from_slice(&fragment.directive_ids);
                        self.parent_type_conditions.push(ty);
                        self.rec_ingest_selection_set(id, fragment.selection_set())?;
                        self.parent_type_conditions.pop();
                        self.parent_directive_ids.truncate(n);
                    } else {
                        let n = self.parent_directive_ids.len();
                        self.parent_directive_ids.extend_from_slice(&fragment.directive_ids);
                        self.rec_ingest_selection_set(id, fragment.selection_set())?;
                        self.parent_directive_ids.truncate(n);
                    }
                }
            }
        }

        Ok(())
    }

    fn can_be_present(&self, id: QuerySelectionSetId, ty: CompositeTypeId) -> bool {
        let parent_output_type = self.builder.query[id].output_type_id;
        if parent_output_type == ty {
            return true;
        }
        let ctx = self.builder.ctx();
        parent_output_type
            .walk(ctx)
            .has_non_empty_intersection_with(ty.walk(ctx))
    }

    fn add_operation_field(
        &mut self,
        selection_set_id: QuerySelectionSetId,
        field: operation::Field<'op>,
    ) -> crate::Result<()> {
        let schema = self.builder.schema;
        let type_conditions = {
            let query = &mut self.builder.query;
            let start = query.shared_type_conditions.len();
            query
                .shared_type_conditions
                .extend_from_slice(&self.parent_type_conditions);
            (start..query.shared_type_conditions.len()).into()
        };
        let flat_directive_id = self.ingest_directives(field.directive_ids());

        let field = match field {
            operation::Field::Data(field) => field,
            operation::Field::Typename(field) => {
                if self.builder.query[selection_set_id]
                    .typename_fields
                    .iter()
                    .all(|existing| {
                        self.builder.query[existing.type_conditions] != self.builder.query[type_conditions]
                            && existing.response_key != field.response_key
                    })
                {
                    let selection_set = &mut self.builder.query.selection_sets[usize::from(selection_set_id)];
                    selection_set.typename_fields.push(QueryTypenameField {
                        type_conditions,
                        response_key: field.response_key,
                    });

                    if selection_set.typename_node_ix.is_none() {
                        let ix = self
                            .builder
                            .query
                            .graph
                            .add_node(SpaceNode::Typename(super::TypenameFieldNode { indispensable: true }));
                        self.builder
                            .query
                            .graph
                            .add_edge(selection_set.parent_node_ix, ix, SpaceEdge::TypenameField);
                        selection_set.typename_node_ix = Some(ix);
                    }
                }

                return Ok(());
            }
        };

        let parent_node_ix = self.builder.query[selection_set_id].parent_node_ix;
        let bloom_bit_mask = 1 << (usize::from(field.response_key) % (usize::BITS - 1) as usize);
        let previous_bloom_filter = self.response_key_bloom_filter;
        self.response_key_bloom_filter |= bloom_bit_mask;

        // Only search for a field with the same response key if we're likely to find one.
        if previous_bloom_filter & bloom_bit_mask != 0 {
            let ctx = OperationContext {
                schema,
                operation: self.builder.operation,
            };
            for node_ix in self
                .builder
                .query
                .graph
                .neighbors_directed(parent_node_ix, Direction::Outgoing)
            {
                let SpaceNode::QueryField(node) = self.builder.query.graph[node_ix] else {
                    continue;
                };
                let query_field = &self.builder.query[node.id];
                if query_field.response_key != Some(field.response_key) {
                    continue;
                }
                if query_field.definition_id == field.definition_id
                    && !are_arguments_equivalent(ctx, query_field.argument_ids, field.argument_ids.into())
                {
                    return Err(crate::Error::InconsistentFieldArguments {
                        name: field.response_key_str().to_string(),
                        location1: query_field.location,
                        location2: field.location,
                    });
                }
                // Merging fields if they have similar type conditions and @skip/@include. We could
                // merge the later, the former is hard.
                if self.builder.query[query_field.type_conditions] == self.builder.query[type_conditions]
                    && query_field.flat_directive_id == flat_directive_id
                {
                    if let Some(id) = query_field.selection_set_id {
                        self.queue.push_back(IngestSelectionSet {
                            id,
                            selection_set: field.selection_set(),
                        });
                    }
                    return Ok(());
                }
            }
        }

        let query_field_id = (self.builder.query.fields.len() - 1).into();
        let query_field_node_ix = self
            .builder
            .push_query_field_node(query_field_id, FieldFlags::INDISPENSABLE);
        self.builder
            .query
            .graph
            .add_edge(parent_node_ix, query_field_node_ix, SpaceEdge::Field);

        let field_definition = field.definition_id.walk(schema);
        let nested_selection_set_id = field_definition
            .ty()
            .definition_id
            .as_composite_type()
            .map(|output_type_id| {
                let selection_set = QuerySelectionSet {
                    parent_node_ix: query_field_node_ix,
                    output_type_id,
                    typename_node_ix: None,
                    typename_fields: Vec::new(),
                };

                self.builder.query.selection_sets.push(selection_set);
                let id = (self.builder.query.selection_sets.len() - 1).into();
                self.queue.push_back(IngestSelectionSet {
                    id,
                    selection_set: field.selection_set(),
                });

                id
            });

        let query_field = QueryField {
            query_position: Some(self.next_query_position()),
            type_conditions,
            response_key: Some(field.response_key),
            subgraph_key: None,
            definition_id: field.definition_id,
            argument_ids: field.argument_ids.into(),
            location: field.location,
            flat_directive_id,
            selection_set_id: nested_selection_set_id,
        };
        self.builder.query.fields.push(query_field);

        let query = &mut self.builder.query;
        for directive in field.definition_id.walk(schema).directives() {
            let TypeSystemDirective::Authorized(auth) = directive else {
                continue;
            };
            if let Some(fields) = auth.fields() {
                self.builder.create_requirement_task_stack.push(CreateRequirementTask {
                    petitioner_field_id: query_field_id,
                    dependent_ix: query_field_node_ix,
                    indispensable: query.graph[query_field_node_ix]
                        .as_query_field()
                        .unwrap()
                        .is_indispensable(),
                    required_field_set: fields,
                    parent_selection_set_id: selection_set_id,
                })
            }
            if let Some((node, selection_set_id)) = auth.node().zip(nested_selection_set_id) {
                self.builder.create_requirement_task_stack.push(CreateRequirementTask {
                    petitioner_field_id: query_field_id,
                    dependent_ix: query_field_node_ix,
                    indispensable: query.graph[query_field_node_ix]
                        .as_query_field()
                        .unwrap()
                        .is_indispensable(),
                    parent_selection_set_id: selection_set_id,
                    required_field_set: node,
                })
            }
        }

        Ok(())
    }

    fn ingest_directives(
        &mut self,
        field_directive_ids: &[ExecutableDirectiveId],
    ) -> Option<DeduplicatedFlatExecutableDirectivesId> {
        if self.parent_directive_ids.is_empty() && field_directive_ids.is_empty() {
            return None;
        }
        let mut directives = Vec::with_capacity(self.parent_directive_ids.len() + field_directive_ids.len());
        directives.extend_from_slice(&self.parent_directive_ids);
        directives.extend_from_slice(field_directive_ids);
        directives.sort_unstable();

        let next_id = self
            .builder
            .query
            .deduplicated_flat_sorted_executable_directives
            .len()
            .into();
        Some(
            *self
                .builder
                .query
                .deduplicated_flat_sorted_executable_directives
                .entry(directives)
                .or_insert(next_id),
        )
    }
}
