use operation::{ExecutableDirectiveId, QueryPosition};
use petgraph::stable_graph::NodeIndex;
use schema::{CompositeTypeId, TypeSystemDirective};
use walker::Walk;

use crate::{FieldFlags, QueryField};

use super::{builder::RawQueryBuilder, providable_fields::CreateRequirementTask, QueryFieldNode, SpaceEdge, SpaceNode};

struct IngestSelectionSet<'op> {
    parent_query_field_ix: NodeIndex,
    parent_output_type: CompositeTypeId,
    selection_set: operation::SelectionSet<'op>,
}

impl<'schema, 'op> RawQueryBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn ingest_operation_fields(&mut self) {
        let stack = vec![IngestSelectionSet {
            parent_query_field_ix: self.query.root_ix,
            parent_output_type: CompositeTypeId::Object(self.operation.root_object_id),
            selection_set: self.ctx().root_selection_set(),
        }];
        OperationFieldsIngestor {
            builder: self,
            stack,
            parent_type_conditions: Vec::new(),
            parent_directive_ids: Vec::new(),
            next_query_position: 0,
        }
        .ingest();
    }
}

struct OperationFieldsIngestor<'schema, 'op, 'builder> {
    builder: &'builder mut RawQueryBuilder<'schema, 'op>,
    stack: Vec<IngestSelectionSet<'op>>,
    parent_type_conditions: Vec<CompositeTypeId>,
    parent_directive_ids: Vec<ExecutableDirectiveId>,
    next_query_position: u16,
}

impl<'schema, 'op, 'builder> OperationFieldsIngestor<'schema, 'op, 'builder>
where
    'schema: 'op,
    'op: 'builder,
{
    fn ingest(&mut self) {
        while let Some(IngestSelectionSet {
            parent_query_field_ix,
            parent_output_type,
            selection_set,
        }) = self.stack.pop()
        {
            self.next_query_position = 0;
            self.parent_type_conditions.clear();
            self.parent_directive_ids.clear();
            self.rec_ingest_selection_set(parent_query_field_ix, parent_output_type, selection_set);
        }
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
    ) {
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
                    self.add_operation_field(parent_query_field_node_ix, parent_output_type, field);
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
                    );
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
                        );
                        self.parent_type_conditions.pop();
                        self.parent_directive_ids.truncate(n);
                    } else {
                        let n = self.parent_directive_ids.len();
                        self.parent_directive_ids.extend_from_slice(&fragment.directive_ids);
                        self.rec_ingest_selection_set(
                            parent_query_field_node_ix,
                            parent_output_type,
                            fragment.selection_set(),
                        );
                        self.parent_directive_ids.truncate(n);
                    }
                }
            }
        }
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
    ) {
        let schema = self.builder.schema;
        let query_position = Some(self.next_query_position());
        let query = &mut self.builder.query;
        let type_conditions = {
            let start = query.shared_type_conditions.len();
            query
                .shared_type_conditions
                .extend_from_slice(&self.parent_type_conditions);
            (start..query.shared_type_conditions.len()).into()
        };
        let query_field = match field {
            operation::Field::Data(field) => QueryField {
                query_position,
                type_conditions,
                key: Some(field.key),
                subgraph_key: None,
                definition_id: Some(field.definition_id),
                argument_ids: field.argument_ids.into(),
                location: field.location,
                directive_ids: {
                    let start = query.shared_directives.len();
                    query.shared_directives.extend_from_slice(&self.parent_directive_ids);
                    query.shared_directives.extend_from_slice(&field.directive_ids);
                    (start..query.shared_directives.len()).into()
                },
            },
            operation::Field::Typename(field) => QueryField {
                query_position,
                type_conditions,
                key: Some(field.key),
                subgraph_key: None,
                definition_id: None,
                argument_ids: Default::default(),
                location: field.location,
                directive_ids: {
                    let start = query.shared_directives.len();
                    query.shared_directives.extend_from_slice(&self.parent_directive_ids);
                    (start..query.shared_directives.len()).into()
                },
            },
        };
        query.fields.push(query_field);
        let query_field_id = (query.fields.len() - 1).into();
        let query_field_node_ix = query.graph.add_node(SpaceNode::QueryField(QueryFieldNode {
            field_id: query_field_id,
            flags: FieldFlags::INDISPENSABLE,
        }));
        if let Some(field_definition) = query[query_field_id].definition_id.walk(schema) {
            query
                .graph
                .add_edge(parent_query_field_node_ix, query_field_node_ix, SpaceEdge::Field);
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
            if let Some(ty) = field_definition.ty().definition_id.as_composite_type() {
                self.stack.push(IngestSelectionSet {
                    parent_query_field_ix: query_field_node_ix,
                    parent_output_type: ty,
                    selection_set: field.selection_set(),
                })
            }
        } else {
            query
                .graph
                .add_edge(parent_query_field_node_ix, query_field_node_ix, SpaceEdge::Field);
        }
    }
}
