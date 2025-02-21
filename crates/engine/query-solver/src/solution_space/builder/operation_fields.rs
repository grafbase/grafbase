use std::{
    collections::{HashMap, VecDeque},
    hash::BuildHasherDefault,
};

use fxhash::FxHasher32;
use operation::{ExecutableDirectiveId, OperationContext, QueryPosition};
use petgraph::{Direction, stable_graph::NodeIndex};
use schema::{CompositeTypeId, TypeSystemDirective};
use walker::Walk;

use crate::{DeduplicatedFlatExecutableDirectivesId, FieldFlags, QueryField, are_arguments_equivalent};

use super::{SpaceEdge, SpaceNode, builder::QuerySolutionSpaceBuilder, providable_fields::CreateRequirementTask};

struct IngestSelectionSet<'op> {
    parent_query_field_node_ix: NodeIndex,
    parent_output_type: CompositeTypeId,
    selection_set: operation::SelectionSet<'op>,
}

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn ingest_operation_fields(&mut self) -> crate::Result<()> {
        let queue = vec![IngestSelectionSet {
            parent_query_field_node_ix: self.query.root_node_ix,
            parent_output_type: CompositeTypeId::Object(self.operation.root_object_id),
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
        let mut selection_set_to_response_key_bloom_filter: HashMap<NodeIndex, usize, BuildHasherDefault<FxHasher32>> =
            HashMap::with_capacity_and_hasher(self.builder.operation.data_fields.len() >> 2, Default::default());
        while let Some(IngestSelectionSet {
            parent_query_field_node_ix,
            parent_output_type,
            selection_set,
        }) = self.queue.pop_front()
        {
            self.parent_type_conditions.clear();
            self.parent_directive_ids.clear();
            let bloom_filter = selection_set_to_response_key_bloom_filter
                .entry(parent_query_field_node_ix)
                .or_default();
            self.response_key_bloom_filter = *bloom_filter;
            self.rec_ingest_selection_set(parent_query_field_node_ix, parent_output_type, selection_set)?;
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
        parent_query_field_node_ix: NodeIndex,
        parent_output_type: CompositeTypeId,
        selection_set: operation::SelectionSet<'op>,
    ) -> crate::Result<()> {
        for selection in selection_set {
            match selection {
                operation::Selection::Field(field) => {
                    if let operation::Field::Data(field) = field {
                        let ty = field.definition().parent_entity_id.as_composite_type();
                        if !self.can_be_present(parent_output_type, ty) {
                            // This field can never appear, likely comes from a common
                            // fragment. In the Operation validation we only verify that fragments have
                            // a common element with their direct parent.
                            continue;
                        }
                    }
                    self.add_operation_field(parent_query_field_node_ix, parent_output_type, field)?;
                }
                operation::Selection::FragmentSpread(spread) => {
                    let fragment = spread.fragment();
                    let ty = fragment.type_condition_id;
                    if !self.can_be_present(parent_output_type, ty) {
                        // This selection can never appear, likely comes from a common
                        // fragment. In the Operation validation we only verify that fragments have
                        // a common element with their direct
                        continue;
                    }

                    let n = self.parent_directive_ids.len();
                    self.parent_directive_ids.extend_from_slice(&spread.directive_ids);
                    self.parent_type_conditions.push(ty);
                    self.rec_ingest_selection_set(
                        parent_query_field_node_ix,
                        parent_output_type,
                        fragment.selection_set(),
                    )?;
                    self.parent_type_conditions.pop();
                    self.parent_directive_ids.truncate(n);
                }
                operation::Selection::InlineFragment(fragment) => {
                    if let Some(ty) = fragment.type_condition_id {
                        if !self.can_be_present(parent_output_type, ty) {
                            // This selection can never appear, likely comes from a common
                            // fragment. In the Operation validation we only verify that fragments have
                            // a common element with their direct
                            continue;
                        }

                        let n = self.parent_directive_ids.len();
                        self.parent_directive_ids.extend_from_slice(&fragment.directive_ids);
                        self.parent_type_conditions.push(ty);
                        self.rec_ingest_selection_set(
                            parent_query_field_node_ix,
                            parent_output_type,
                            fragment.selection_set(),
                        )?;
                        self.parent_type_conditions.pop();
                        self.parent_directive_ids.truncate(n);
                    } else {
                        let n = self.parent_directive_ids.len();
                        self.parent_directive_ids.extend_from_slice(&fragment.directive_ids);
                        self.rec_ingest_selection_set(
                            parent_query_field_node_ix,
                            parent_output_type,
                            fragment.selection_set(),
                        )?;
                        self.parent_directive_ids.truncate(n);
                    }
                }
            }
        }

        Ok(())
    }

    fn can_be_present(&self, parent_output_type: CompositeTypeId, ty: CompositeTypeId) -> bool {
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
        parent_query_field_node_ix: NodeIndex,
        parent_output_type: CompositeTypeId,
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
        let response_key = field.response_key();
        let (definition_id, output_ty) = field.definition().map(|def| (def.id, def.ty())).unzip();

        let bloom_bit_mask = 1 << (usize::from(response_key) % (usize::BITS - 1) as usize);
        let mut existing_query_field_node_ix = None;
        // Only search for a field with the same response key if we're likely to find one.
        if self.response_key_bloom_filter & bloom_bit_mask != 0 {
            let ctx = OperationContext {
                schema,
                operation: self.builder.operation,
            };
            for node_ix in self
                .builder
                .query
                .graph
                .neighbors_directed(parent_query_field_node_ix, Direction::Outgoing)
            {
                let SpaceNode::QueryField(node) = self.builder.query.graph[node_ix] else {
                    continue;
                };
                let query_field = &self.builder.query[node.id];
                if query_field.response_key != Some(response_key) {
                    continue;
                }
                if query_field.definition_id == definition_id
                    && !are_arguments_equivalent(
                        ctx,
                        query_field.argument_ids,
                        field.as_data().map(|f| f.argument_ids.into()).unwrap_or_default(),
                    )
                {
                    return Err(crate::Error::InconsistentFieldArguments {
                        name: field.response_key_str().to_string(),
                        location1: query_field.location,
                        location2: field.location(),
                    });
                }
                // Merging fields if they have similar type conditions and @skip/@include. We could
                // merge the later, the former is hard.
                if self.builder.query[query_field.type_conditions] == self.builder.query[type_conditions]
                    && query_field.flat_directive_id == flat_directive_id
                {
                    existing_query_field_node_ix = Some(node_ix);
                    break;
                }
            }
        }
        self.response_key_bloom_filter |= bloom_bit_mask;

        let query_field_node_ix = existing_query_field_node_ix.unwrap_or_else(|| {
            let (query_field_id, edge_weight) = {
                let (query_field, edge_weight) = match field {
                    operation::Field::Data(field) => (
                        QueryField {
                            query_position: Some(self.next_query_position()),
                            type_conditions,
                            response_key: Some(field.response_key),
                            subgraph_key: None,
                            definition_id,
                            argument_ids: field.argument_ids.into(),
                            location: field.location,
                            flat_directive_id,
                        },
                        SpaceEdge::Field,
                    ),
                    operation::Field::Typename(field) => (
                        QueryField {
                            query_position: Some(self.next_query_position()),
                            type_conditions,
                            response_key: Some(field.response_key),
                            subgraph_key: None,
                            definition_id: None,
                            argument_ids: Default::default(),
                            location: field.location,
                            flat_directive_id,
                        },
                        SpaceEdge::TypenameField,
                    ),
                };
                self.builder.query.fields.push(query_field);
                ((self.builder.query.fields.len() - 1).into(), edge_weight)
            };
            let query_field_node_ix = self
                .builder
                .push_query_field_node(query_field_id, FieldFlags::INDISPENSABLE);
            self.builder
                .query
                .graph
                .add_edge(parent_query_field_node_ix, query_field_node_ix, edge_weight);

            let query = &mut self.builder.query;
            if let Some(field_definition) = query[query_field_id].definition_id.walk(schema) {
                for directive in field_definition.directives() {
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
                            parent_query_field_node_ix,
                            parent_output_type,
                        })
                    }
                    if let Some((node, output_type)) =
                        auth.node().zip(field_definition.ty().definition_id.as_composite_type())
                    {
                        self.builder.create_requirement_task_stack.push(CreateRequirementTask {
                            petitioner_field_id: query_field_id,
                            dependent_ix: query_field_node_ix,
                            indispensable: query.graph[query_field_node_ix]
                                .as_query_field()
                                .unwrap()
                                .is_indispensable(),
                            parent_query_field_node_ix: query_field_node_ix,
                            parent_output_type: output_type,
                            required_field_set: node,
                        })
                    }
                }
            }

            query_field_node_ix
        });

        if let Some(ty) = output_ty.and_then(|ty| ty.definition_id.as_composite_type()) {
            self.queue.push_back(IngestSelectionSet {
                parent_query_field_node_ix: query_field_node_ix,
                parent_output_type: ty,
                selection_set: field.selection_set(),
            })
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
