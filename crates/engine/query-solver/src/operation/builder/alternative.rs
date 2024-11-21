use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction};
use schema::{CompositeType, FieldDefinition};
use walker::Walk;

use super::{builder::OperationGraphBuilder, Edge, Operation, QueryField};

impl<'ctx, Op: Operation> OperationGraphBuilder<'ctx, Op> {
    pub(super) fn try_providing_an_alternative_field(&mut self, query_field_ix: NodeIndex) -> bool {
        let Some((parent_query_field_ix, parent_query_node)) = self
            .graph
            .edges_directed(query_field_ix, Direction::Incoming)
            .filter(|edge| matches!(edge.weight(), Edge::Field))
            .filter_map(|edge| {
                self.graph[edge.source()]
                    .as_query_field()
                    .map(|node| (edge.source(), node))
            })
            .next()
        else {
            return false;
        };

        let Some(parent_output) = self
            .operation
            .field_definition(parent_query_node.id)
            .walk(self.schema)
            .and_then(|def| def.ty().definition().as_composite_type())
        else {
            return false;
        };

        let Some((node, field_definition)) = self.graph[query_field_ix].as_query_field().and_then(|node| {
            self.operation
                .field_definition(node.id)
                .map(|def| (*node, def.walk(self.schema)))
        }) else {
            return false;
        };

        tracing::debug!(
            "Trying to find alternative for field {}.{}",
            field_definition.parent_entity().name(),
            field_definition.name()
        );
        self.try_providing_field_through_interface(
            parent_output,
            parent_query_field_ix,
            query_field_ix,
            node,
            field_definition,
        )
    }

    fn try_providing_field_through_interface(
        &mut self,
        parent_output: CompositeType<'ctx>,
        parent_query_field_ix: NodeIndex,
        existing_query_field_ix: NodeIndex,
        existing_node: QueryField<Op::FieldId>,
        field_definition: FieldDefinition<'ctx>,
    ) -> bool {
        tracing::debug!("Trying to provide field through interface.");
        let implemented_interfaces = field_definition.parent_entity().interface_ids();

        // If parent is an implemented interface.
        if let Some(interface) = parent_output
            .as_interface()
            .filter(|inf| implemented_interfaces.contains(&inf.id))
        {
            if let Some(interface_field_definition) = interface.find_field_by_name(field_definition.name()) {
                // FIXME: Should not add extra field if the interface field is already present.
                let new_field_id = self
                    .operation
                    .create_potential_extra_interface_field_alternative(existing_node.id, interface_field_definition);
                let new_query_field_ix = self.push_query_field(new_field_id, existing_node.flags);
                if self.could_provide_new_field(parent_query_field_ix, new_field_id) {
                    self.replace_query_node(existing_query_field_ix, new_query_field_ix);
                    return true;
                }
            }
        }

        false
    }

    fn replace_query_node(&mut self, existing: NodeIndex, new: NodeIndex) {
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
        self.graph.remove_node(existing);
    }

    fn could_provide_new_field(&mut self, parent_query_field_ix: NodeIndex, field_id: Op::FieldId) -> bool {
        self.push_field_to_provide(parent_query_field_ix, field_id);
        self.loop_over_ingestion_stacks();
        self.providable_fields_bitset[field_id.into()]
    }
}
