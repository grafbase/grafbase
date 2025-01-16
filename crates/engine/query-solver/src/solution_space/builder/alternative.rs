use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};
use schema::{CompositeType, FieldDefinition};
use walker::Walk;

use crate::{NodeFlags, QueryField, QueryFieldId, QuerySelectionSetId};

use super::{builder::QuerySolutionSpaceBuilder, providable_fields::UnplannableField, SpaceEdge, SpaceNode};

impl<'schema, 'op> QuerySolutionSpaceBuilder<'schema, 'op>
where
    'schema: 'op,
{
    pub(super) fn handle_unplannable_field(
        &mut self,
        UnplannableField {
            parent_selection_set_id,
            node_ix,
        }: UnplannableField,
    ) -> crate::Result<()> {
        match self.query.graph[node_ix] {
            SpaceNode::Typename { flags } if !flags.contains(NodeFlags::PROVIDABLE) => {
                return Ok(());
            }
            SpaceNode::QueryField { id, flags } if !flags.contains(NodeFlags::PROVIDABLE) => {
                if flags.contains(NodeFlags::UNREACHABLE) {
                    let mut stack = vec![node_ix];
                    while let Some(id) = stack.pop() {
                        stack.extend(self.query.graph.neighbors_directed(id, Direction::Outgoing));
                        self.query.graph.remove_node(id);
                    }
                    return Ok(());
                }
            }
            _ => (),
        }
        let SpaceNode::QueryField {
            id: query_field_id,
            flags,
        } = self.query.graph[query_field_node_ix]
        else {
            return Ok(());
        };

        let parent_query_field_node_ix = self.query[parent_selection_set_id].parent_node_ix;
        let SpaceNode::QueryField {
            id: parent_query_field_id,
            ..
        } = self.query.graph[parent_query_field_node_ix]
        else {
            return Ok(());
        };

        if !self.try_providing_an_alternative_field(
            parent_selection_set_id,
            parent_query_field_node_ix,
            parent_query_field_id,
            query_field_node_ix,
            query_field_id,
        ) {
            tracing::debug!("Unplannable Query:\n{}", self.query.to_pretty_dot_graph(self.ctx()));

            let definition = self.query[query_field_id].definition_id.walk(self.schema);
            let name = format!("{}.{}", definition.parent_entity().name(), definition.name());
            return Err(crate::Error::CouldNotPlanField { name });
        };

        Ok(())
    }

    pub(super) fn try_providing_an_alternative_field(
        &mut self,
        parent_selection_set_id: QuerySelectionSetId,
        parent_query_field_node_ix: NodeIndex,
        parent_query_field_id: QueryFieldId,
        query_field_node_ix: NodeIndex,
        query_field_id: QueryFieldId,
    ) -> bool {
        let Some(parent_output) = self.query[parent_query_field_id]
            .definition_id
            .walk(self.schema)
            .ty()
            .definition()
            .as_composite_type()
        else {
            return false;
        };

        let field_definition = self.query[query_field_id].definition_id.walk(self.schema);

        tracing::debug!(
            "Trying to find alternative for field {}.{}",
            field_definition.parent_entity().name(),
            field_definition.name()
        );

        if self.try_providing_field_through_interface(
            parent_selection_set_id,
            parent_output,
            query_field_node_ix,
            query_field_id,
            field_definition,
        ) {
            return true;
        }

        if self.try_providing_interface_field_through_implementors(
            parent_selection_set_id,
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
        parent_selection_set_id: QuerySelectionSetId,
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
                .flags()
                .unwrap_or_default();

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
                            definition_id: object_field_definition.id,
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

                        if !self.could_provide_new_field(parent_selection_set_id, new_query_field_node_ix, new_field_id)
                        {
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
        parent_selection_set_id: QuerySelectionSetId,
        parent_output: CompositeType<'schema>,
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
                self.query[existing_query_field_id].definition_id = interface_field_definition.id;

                if self.could_provide_new_field(
                    parent_selection_set_id,
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
        parent_selection_set_id: QuerySelectionSetId,
        query_field_node_ix: NodeIndex,
        query_field_id: QueryFieldId,
    ) -> bool {
        self.create_providable_fields_task_for_new_field(parent_selection_set_id, query_field_node_ix, query_field_id);
        self.loop_over_tasks();
        self.query.graph[query_field_node_ix]
            .flags()
            .unwrap()
            .contains(NodeFlags::PROVIDABLE)
    }
}
