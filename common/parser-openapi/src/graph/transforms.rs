//! We can figure out a lot of the transformations between our OpenAPI and the eventual
//! schema in code as part of the typed layer we build on top of the graph.
//!
//! But some of these are quite tricky to implement there so we apply them directly
//! to the graph prior to building the typed layer.

use petgraph::{visit::EdgeRef, Direction};

use super::{Edge, Node, OpenApiGraph};

/// OpenAPI allows for unions of any type but in GraphQL unions can only contain
/// objects.  Here we handle that by converting any fields containing impossible
/// unions into JSON fields.
pub fn impossible_unions_to_json(graph: &mut OpenApiGraph) {
    let unions = graph
        .graph
        .node_indices()
        .filter(|index| matches!(graph.graph[*index], Node::Union))
        .collect::<Vec<_>>();

    for index in unions {
        let output_type = super::OutputType::from_index(index, graph).expect("we've already filtered to unions");
        if output_type.possible_types(graph).is_empty() {
            // This is an invalid union - it probably only contains scalars.
            // Lets just make any field containing it into JSON

            let type_edges = graph
                .graph
                .edges_directed(index, Direction::Incoming)
                .filter_map(|edge| match edge.weight() {
                    Edge::HasType { .. } | Edge::HasResponseType { .. } | Edge::HasField { .. } => Some(edge.id()),
                    _ => None,
                })
                .collect::<Vec<_>>();

            if type_edges.is_empty() {
                continue;
            }

            let json_node = graph.graph.add_node(Node::Scalar(super::ScalarKind::JsonObject));

            for type_edge in type_edges {
                let (source_node, _) = graph.graph.edge_endpoints(type_edge).unwrap();
                match graph.graph.remove_edge(type_edge).unwrap() {
                    Edge::HasType { wrapping } => {
                        graph.graph.add_edge(source_node, json_node, Edge::HasType { wrapping });
                    }
                    Edge::HasField { name, wrapping } => {
                        graph
                            .graph
                            .add_edge(source_node, json_node, Edge::HasField { name, wrapping });
                    }
                    Edge::HasResponseType {
                        status_code,
                        content_type,
                        wrapping,
                    } => {
                        graph.graph.add_edge(
                            source_node,
                            json_node,
                            Edge::HasResponseType {
                                status_code,
                                content_type,
                                wrapping,
                            },
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
