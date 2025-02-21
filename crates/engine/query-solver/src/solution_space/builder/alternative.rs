use petgraph::{Direction, stable_graph::NodeIndex, visit::EdgeRef};
use schema::{CompositeType, CompositeTypeId, FieldDefinition};
use walker::Walk;

use crate::{FieldFlags, QueryField, QueryFieldId};

use super::{
    QueryFieldNode, SpaceEdge, SpaceNode, builder::QuerySolutionSpaceBuilder, providable_fields::UnplannableField,
};

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn handle_unplannable_field(
        &mut self,
        UnplannableField {
            parent_query_field_node_ix,
            query_field_node_ix,
        }: UnplannableField,
    ) -> crate::Result<()> {
        let SpaceNode::QueryField(QueryFieldNode {
            id: query_field_id,
            flags,
        }) = self.query.graph[query_field_node_ix]
        else {
            return Ok(());
        };
        if flags.contains(FieldFlags::UNREACHABLE) {
            if self
                .query
                .graph
                .edges_directed(query_field_node_ix, Direction::Incoming)
                .any(|edge| matches!(edge.weight(), SpaceEdge::Provides))
            {
                let SpaceNode::QueryField(QueryFieldNode { flags, .. }) = &mut self.query.graph[query_field_node_ix]
                else {
                    return Ok(());
                };
                flags.remove(FieldFlags::UNREACHABLE);
                return Ok(());
            }
            let mut stack = vec![query_field_node_ix];
            while let Some(id) = stack.pop() {
                stack.extend(self.query.graph.neighbors_directed(id, Direction::Outgoing));
                self.query.graph.remove_node(id);
            }
            return Ok(());
        }
        // FIXME: Should only check for indispensable fields. And then if we add new indispensable
        // field ensure we can provide them instead of everything at once...
        if flags.contains(FieldFlags::PROVIDABLE) {
            return Ok(());
        }

        if !self.query.graph[parent_query_field_node_ix]
            .as_query_field()
            .copied()
            .map(|parent| {
                self.try_providing_an_alternative_field(
                    parent_query_field_node_ix,
                    parent.id,
                    query_field_node_ix,
                    query_field_id,
                )
            })
            .unwrap_or_default()
        {
            tracing::debug!("Unplannable Query:\n{}", self.query.to_pretty_dot_graph(self.ctx()));

            return Err(crate::Error::CouldNotPlanField {
                name: self.query[query_field_id]
                    .definition_id
                    .walk(self.schema)
                    .map(|def| {
                        tracing::debug!("Could not plan field:\n{def:#?}");
                        format!("{}.{}", def.parent_entity().name(), def.name())
                    })
                    .unwrap_or("__typename".into()),
            });
        };

        Ok(())
    }

    pub(super) fn try_providing_an_alternative_field(
        &mut self,
        parent_query_field_node_ix: NodeIndex,
        parent_query_field_id: QueryFieldId,
        query_field_node_ix: NodeIndex,
        query_field_id: QueryFieldId,
    ) -> bool {
        let Some(parent_output) = self.query[parent_query_field_id]
            .definition_id
            .walk(self.schema)
            .and_then(|def| def.ty().definition().as_composite_type())
        else {
            return false;
        };

        let Some(field_definition) = self.query[query_field_id].definition_id.walk(self.schema) else {
            return false;
        };

        tracing::debug!(
            "Trying to find alternative for field {}.{}",
            field_definition.parent_entity().name(),
            field_definition.name()
        );

        if self.try_providing_field_through_interface(
            parent_output,
            parent_query_field_node_ix,
            query_field_node_ix,
            query_field_id,
            field_definition,
        ) {
            return true;
        }

        if self.try_providing_interface_field_through_implementors(
            parent_output,
            parent_query_field_node_ix,
            query_field_node_ix,
            query_field_id,
            field_definition,
        ) {
            return true;
        }

        false
    }

    fn try_providing_interface_field_through_implementors(
        &mut self,
        parent_output: CompositeType<'schema>,
        parent_query_field_node_ix: NodeIndex,
        existing_query_field_node_ix: NodeIndex,
        existing_query_field_id: QueryFieldId,
        field_definition: FieldDefinition<'schema>,
    ) -> bool {
        let Some(interface) = field_definition.parent_entity().as_interface() else {
            return false;
        };

        let mut subgraph_ids = self
            .query
            .graph
            .edges_directed(parent_query_field_node_ix, Direction::Incoming)
            .filter_map(|edge| {
                if matches!(edge.weight(), SpaceEdge::Provides) {
                    self.query.graph[edge.source()].as_providable_field()
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
            let left = parent_output.possible_type_ids();
            let right = &interface.possible_type_ids;
            let mut l = 0;
            let mut r = 0;

            let existing_flags = self.query.graph[existing_query_field_node_ix]
                .as_query_field()
                .unwrap()
                .flags;

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
                        if object_field_definition.exists_in_subgraph_ids.is_empty() {
                            return false;
                        }

                        self.query.fields.push(QueryField {
                            definition_id: Some(object_field_definition.id),
                            ..self.query[existing_query_field_id].clone()
                        });
                        let new_field_id = QueryFieldId::from(self.query.fields.len() - 1);
                        let new_query_field_node_ix = self.push_query_field_node(new_field_id, existing_flags);
                        self.query.graph.add_edge(
                            parent_query_field_node_ix,
                            new_query_field_node_ix,
                            SpaceEdge::Field,
                        );
                        self.deep_copy_query_field_nodes(existing_query_field_node_ix, new_query_field_node_ix);

                        if !self.could_provide_new_field(
                            parent_query_field_node_ix,
                            parent_output.id(),
                            new_query_field_node_ix,
                            new_field_id,
                        ) {
                            return false;
                        }
                        found_alternative = true;

                        l += 1;
                        r += 1;
                    }
                }
            }

            if found_alternative {
                // Removing original fields. We have no choice but to keep them intact until the
                // end to only copy the original edges to new object field.
                let mut stack = vec![existing_query_field_node_ix];
                while let Some(id) = stack.pop() {
                    stack.extend(self.query.graph.neighbors_directed(id, Direction::Outgoing));
                    self.query.graph.remove_node(id);
                }
            }

            // If there is no intersection between the parent output type and the interface, we
            // should have never tried to plan this field at all. So if we reach that point and
            // couldn't find any alternative, it's a planning error.
            return found_alternative;
        }
        // TODO: handle other cases?

        false
    }

    fn try_providing_field_through_interface(
        &mut self,
        parent_output: CompositeType<'schema>,
        parent_query_field_node_ix: NodeIndex,
        existing_query_field_node_ix: NodeIndex,
        existing_query_field_id: QueryFieldId,
        field_definition: FieldDefinition<'schema>,
    ) -> bool {
        let implemented_interfaces = field_definition.parent_entity().interface_ids();

        // If parent is an implemented interface.
        if let Some(interface) = parent_output
            .as_interface()
            .filter(|inf| implemented_interfaces.contains(&inf.id))
        {
            if let Some(interface_field_definition) = interface.find_field_by_name(field_definition.name()) {
                // FIXME: Should not keep field if the interface field is already present.
                self.query[existing_query_field_id].definition_id = Some(interface_field_definition.id);

                if self.could_provide_new_field(
                    parent_query_field_node_ix,
                    parent_output.id(),
                    existing_query_field_node_ix,
                    existing_query_field_id,
                ) {
                    return true;
                }
            }
        }

        false
    }

    fn deep_copy_query_field_nodes(
        &mut self,
        existing_query_field_node_ix: NodeIndex,
        new_query_field_node_ix: NodeIndex,
    ) {
        let mut stack = vec![(existing_query_field_node_ix, new_query_field_node_ix)];
        while let Some((existing_node_ix, new_node_ix)) = stack.pop() {
            let mut incoming_edges = self
                .query
                .graph
                .neighbors_directed(existing_node_ix, Direction::Incoming)
                .detach();
            while let Some((edge_ix, source)) = incoming_edges.next(&self.query.graph) {
                let weight = self.query.graph[edge_ix];
                if matches!(weight, SpaceEdge::Requires) {
                    self.query.graph.add_edge(source, new_node_ix, weight);
                }
            }
            let mut outgoing_edges = self
                .query
                .graph
                .neighbors_directed(existing_node_ix, Direction::Outgoing)
                .detach();
            while let Some((existing_edge_ix, existing_target)) = outgoing_edges.next(&self.query.graph) {
                debug_assert!(matches!(
                    self.query.graph[existing_edge_ix],
                    SpaceEdge::Field | SpaceEdge::TypenameField
                ));
                let new_target = self.query.graph.add_node(self.query.graph[existing_target].clone());
                self.query
                    .graph
                    .add_edge(new_node_ix, new_target, self.query.graph[existing_edge_ix]);
                stack.push((existing_target, new_target));
            }
        }
    }

    fn could_provide_new_field(
        &mut self,
        parent_query_field_node_ix: NodeIndex,
        parent_output_type: CompositeTypeId,
        query_field_node_ix: NodeIndex,
        query_field_id: QueryFieldId,
    ) -> bool {
        self.create_providable_fields_task_for_new_field(
            parent_query_field_node_ix,
            parent_output_type,
            query_field_node_ix,
            query_field_id,
        );
        self.loop_over_tasks();
        self.query.graph[query_field_node_ix]
            .as_query_field()
            .unwrap()
            .flags
            .contains(FieldFlags::PROVIDABLE)
    }
}
