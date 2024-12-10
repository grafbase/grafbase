use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};
use schema::{CompositeType, FieldDefinition};
use walker::Walk;

use crate::FieldFlags;

use super::{builder::OperationGraphBuilder, Edge, Node, Operation, QueryField};

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn try_providing_missing_fields_through_alternatives(&mut self) {
        let mut stack = self
            .graph
            .edges_directed(self.root_ix, Direction::Outgoing)
            .filter(|edge| matches!(edge.weight(), Edge::Field))
            .map(|edge| edge.target())
            .collect::<Vec<_>>();

        while let Some(parent_query_field_ix) = stack.pop() {
            let Some(parent_query_field) = self
                .graph
                .node_weight(parent_query_field_ix)
                .and_then(|node| node.as_query_field())
                .copied()
            else {
                continue;
            };

            let mut i = stack.len();
            stack.extend(
                self.graph
                    .edges_directed(parent_query_field_ix, Direction::Outgoing)
                    .filter(|edge| matches!(edge.weight(), Edge::Field))
                    .map(|edge| edge.target()),
            );
            let n = stack.len();
            while let Some(query_field_ix) = stack.get(i) {
                if let Some(query_field) = self.graph[*query_field_ix].as_query_field().copied() {
                    if !self.providable_fields_bitset[query_field.id.into()]
                        && !self.try_providing_an_alternative_field(
                            parent_query_field_ix,
                            parent_query_field,
                            *query_field_ix,
                            query_field,
                            &mut stack,
                        )
                    {
                        // There's no point continuing. Today we fail if any node is non-providable.
                        return;
                    }
                };
                if i < n {
                    i += 1
                } else {
                    break;
                }
            }
        }
    }

    fn try_providing_an_alternative_field(
        &mut self,
        parent_query_field_ix: NodeIndex,
        parent_query_field: QueryField<Op::FieldId>,
        query_field_ix: NodeIndex,
        query_field: QueryField<Op::FieldId>,
        new_nodes_stack: &mut Vec<NodeIndex>,
    ) -> bool {
        let Some(parent_output) = self
            .operation
            .field_definition(parent_query_field.id)
            .walk(self.schema)
            .and_then(|def| def.ty().definition().as_composite_type())
        else {
            return false;
        };

        let Some(field_definition) = self.operation.field_definition(query_field.id).walk(self.schema) else {
            return false;
        };

        tracing::debug!(
            "Trying to find alternative for field {}.{}",
            field_definition.parent_entity().name(),
            field_definition.name()
        );

        if self.try_providing_field_through_interface(
            parent_output,
            parent_query_field_ix,
            query_field_ix,
            query_field,
            field_definition,
            new_nodes_stack,
        ) {
            return true;
        }

        if self.try_providing_interface_field_through_implementors(
            parent_output,
            parent_query_field_ix,
            query_field_ix,
            query_field,
            field_definition,
            new_nodes_stack,
        ) {
            return true;
        }

        false
    }

    fn try_providing_interface_field_through_implementors(
        &mut self,
        parent_output: CompositeType<'ctx>,
        parent_query_field_ix: NodeIndex,
        existing_query_field_ix: NodeIndex,
        existing_node: QueryField<Op::FieldId>,
        field_definition: FieldDefinition<'ctx>,
        new_nodes_stack: &mut Vec<NodeIndex>,
    ) -> bool {
        let Some(interface) = field_definition.parent_entity().as_interface() else {
            return false;
        };

        let mut subgraph_ids = self
            .graph
            .edges_directed(parent_query_field_ix, Direction::Incoming)
            .filter_map(|edge| {
                if matches!(edge.weight(), Edge::Provides) {
                    self.graph[edge.source()].as_providable_field()
                } else {
                    None
                }
            })
            .map(|node| node.subgraph_id())
            .collect::<Vec<_>>();
        subgraph_ids.sort_unstable();
        subgraph_ids.dedup();

        if subgraph_ids.len() == 1 {
            let subgraph_id = subgraph_ids[0];
            let left = parent_output.possible_types();
            let right = &interface.possible_type_ids;
            let mut l = 0;
            let mut r = 0;

            let mut found_alternative = false;
            while let Some((left_id, right_id)) = left.get(l).copied().zip(right.get(r).copied()) {
                match left_id.cmp(&right_id) {
                    std::cmp::Ordering::Less => l += 1,
                    std::cmp::Ordering::Greater => r += 1,
                    std::cmp::Ordering::Equal => {
                        let object = right[r].walk(self.schema);
                        if !object.implements_interface_in_subgraph(&subgraph_id, &interface.id) {
                            continue;
                        }
                        let object_field_definition = object
                            .find_field_by_name(field_definition.name())
                            .expect("Implements interface but doesn't have its fields?");

                        // Object field is not resolvable by itself. We need to go through the
                        // interface which we can't provide (otherwise we wouldn't be here). So
                        // it's a planning error.
                        if object_field_definition.resolvable_in_ids.is_empty() {
                            return false;
                        }

                        let new_field_id = self.operation.create_potential_alternative_with_different_definition(
                            existing_node.id,
                            object_field_definition,
                            true,
                        );

                        // We used deep clone, so we also need to add nested fields.
                        self.operation
                            .field_ids()
                            .skip(self.field_nodes.len())
                            .for_each(|field_id| {
                                self.push_query_field(field_id, FieldFlags::INDISPENSABLE);
                            });

                        let new_query_field_ix = self[new_field_id];
                        if self.could_provide_new_field(parent_query_field_ix, new_field_id) {
                            let Node::QueryField(new_query_field) = &mut self.graph[new_query_field_ix] else {
                                unreachable!()
                            };
                            new_query_field.flags = existing_node.flags;
                            self.copy_edges(existing_query_field_ix, new_query_field_ix);
                            new_nodes_stack.push(new_query_field_ix);
                            found_alternative = true;
                        } else {
                            return false;
                        }

                        l += 1;
                        r += 1;
                    }
                }
            }

            // If there is no intersection between the parent output type and the interface, we
            // should have never tried to plan this field at all. So if we reach that point and
            // couldn't find any alternative, it's a planning error.
            if !found_alternative {
                return false;
            }

            self.graph.remove_node(existing_query_field_ix);
            self.deleted_fields_bitset.put(existing_node.id.into());

            let mut stack = self.operation.subselection(existing_node.id).collect::<Vec<_>>();
            while let Some(id) = stack.pop() {
                if self
                    .graph
                    .edges_directed(self[id], Direction::Incoming)
                    .any(|edge| matches!(edge.weight(), Edge::Requires))
                {
                    tracing::error!("Can't migrate requirements on nested fields today...");
                    return false;
                }
                stack.extend(self.operation.subselection(id));
                self.graph.remove_node(self[id]);
                self.deleted_fields_bitset.put(id.into());
            }
            return true;
        }
        // TODO: handle other cases?

        false
    }

    fn try_providing_field_through_interface(
        &mut self,
        parent_output: CompositeType<'ctx>,
        parent_query_field_ix: NodeIndex,
        existing_query_field_ix: NodeIndex,
        existing_node: QueryField<Op::FieldId>,
        field_definition: FieldDefinition<'ctx>,
        new_nodes_stack: &mut Vec<NodeIndex>,
    ) -> bool {
        let implemented_interfaces = field_definition.parent_entity().interface_ids();

        // If parent is an implemented interface.
        if let Some(interface) = parent_output
            .as_interface()
            .filter(|inf| implemented_interfaces.contains(&inf.id))
        {
            if let Some(interface_field_definition) = interface.find_field_by_name(field_definition.name()) {
                // FIXME: Should not add extra field if the interface field is already present.
                let new_field_id = self.operation.create_potential_alternative_with_different_definition(
                    existing_node.id,
                    interface_field_definition,
                    false,
                );
                let new_query_field_ix = self.push_query_field(new_field_id, existing_node.flags);
                if self.could_provide_new_field(parent_query_field_ix, new_field_id) {
                    self.copy_edges(existing_query_field_ix, new_query_field_ix);
                    self.graph.remove_node(existing_query_field_ix);
                    self.deleted_fields_bitset.put(existing_node.id.into());
                    new_nodes_stack.push(new_query_field_ix);
                    return true;
                }
            }
        }

        false
    }

    fn copy_edges(&mut self, existing: NodeIndex, new: NodeIndex) {
        let mut edges = self.graph.neighbors_directed(existing, Direction::Outgoing).detach();
        while let Some((edge_ix, target)) = edges.next(&self.graph) {
            self.graph.add_edge(new, target, self.graph[edge_ix]);
        }
        let mut edges = self.graph.neighbors_directed(existing, Direction::Incoming).detach();
        while let Some((edge_ix, source)) = edges.next(&self.graph) {
            // Field edge is already added as we try to provide the new field.
            if self.graph[edge_ix] != Edge::Field {
                self.graph.add_edge(source, new, self.graph[edge_ix]);
            }
        }
    }

    fn could_provide_new_field(&mut self, parent_query_field_ix: NodeIndex, field_id: Op::FieldId) -> bool {
        self.push_field_to_provide(parent_query_field_ix, field_id);
        self.loop_over_ingestion_stacks();
        self.providable_fields_bitset[field_id.into()]
    }
}
