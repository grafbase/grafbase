use petgraph::{
    graph::NodeIndex,
    visit::{Dfs, EdgeFiltered, Walker},
};

use crate::graph::{Arity, Edge, Node};

/// Determines the relationships between operations and resources and inserts edges
/// into the Graph accordingly
pub fn determine_resource_relationships(ctx: &mut super::Context) {
    for operation_index in &ctx.operation_indices {
        let Some(schema_index) = find_operation_resource_schema(ctx, *operation_index) else {
            continue;
        };
        let Some(arity) = schema_arity(ctx, *operation_index, schema_index) else {
            continue;
        };
        ctx.graph
            .add_edge(*operation_index, schema_index, Edge::ForResource { arity });
    }
}

/// Traverses an operations response looking for a single schema reference on the assumption that
/// this schema is probably the primary resource the operation is concerned with.
fn find_operation_resource_schema(ctx: &super::Context, operation_index: NodeIndex) -> Option<NodeIndex> {
    // We only want to look in the response fields
    let filtered_graph = EdgeFiltered::from_fn(&ctx.graph, |edge| {
        matches!(edge.weight(), Edge::HasResponseType { status_code, .. } if status_code.is_success())
            || matches!(edge.weight(), Edge::HasField { .. })
    });

    let mut schema_indices = Dfs::new(&filtered_graph, operation_index)
        .iter(&filtered_graph)
        .filter(|node_index| matches!(&ctx.graph[*node_index], Node::Schema(_)))
        .collect::<Vec<_>>();

    if schema_indices.len() != 1 {
        // If there's more than one schema it's really hard to determine which schema is the "resource"
        // this operation represents, so we just skip this.
        return None;
    }

    schema_indices.pop()
}

// Determines the arity of a schema within an operation
fn schema_arity(ctx: &super::Context, operation_index: NodeIndex, schema_index: NodeIndex) -> Option<Arity> {
    // We only want to look in the response fields
    let filtered_graph = EdgeFiltered::from_fn(&ctx.graph, |edge| {
        matches!(edge.weight(), Edge::HasResponseType { .. } | Edge::HasField { .. })
    });

    // To determine the arity of a schema within an operation we need to know the
    // path between them.
    let (_, mut path) = petgraph::algo::astar(
        &filtered_graph,
        operation_index,
        |current_index| current_index == schema_index,
        |_| 0,
        |_| 0,
    )?;

    // We already have the index of the final node, so we can ditch the end of the path
    path.pop()?;

    // We need to get the Wrapping type from the edge between the schema and its underlying type
    let wrapping_type = ctx.graph.edges(schema_index).find_map(|edge| match edge.weight() {
        Edge::HasType { wrapping, .. } => Some(wrapping.clone()),
        _ => None,
    })?;

    // We also need to take the wrapping type from the edge that points to our schema.
    let schema_parent_index = path.pop()?;
    let outer_wrapping_type = ctx
        .graph
        .edges_connecting(schema_parent_index, schema_index)
        .find_map(|edge| match edge.weight() {
            Edge::HasResponseType { wrapping, .. } | Edge::HasField { wrapping, .. } => Some(wrapping.clone()),
            _ => None,
        })
        .expect("Handle this as well.");

    // Combine the two wrapping types to get the actual arity of the schema
    wrapping_type.wrap_with(outer_wrapping_type).arity()
}
