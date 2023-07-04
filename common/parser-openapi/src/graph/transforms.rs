//! We can figure out a lot of the transformations between our OpenAPI and the eventual
//! schema in code as part of the typed layer we build on top of the graph.
//!
//! But some of these are quite tricky to implement there so we apply them directly
//! to the graph prior to building the typed layer.

use std::collections::HashMap;

use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

use crate::Error;

use super::{all_of_member::AllOfMember, Edge, Node, OpenApiGraph};

pub fn run(graph: &mut OpenApiGraph) -> Result<(), Error> {
    merge_all_of_schemas(graph)?;
    impossible_unions_to_json(graph);
    wrap_scalar_union_variants(graph);

    Ok(())
}

/// OpenAPI allows for unions of any type but in GraphQL unions can only contain
/// objects.  Here we handle that by converting any fields containing impossible
/// unions into JSON fields.
fn impossible_unions_to_json(graph: &mut OpenApiGraph) {
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

/// OpenAPI unions can contain any type but GraphQL unions can only contain
/// Objects.  So we take any remaining unions (after the above transform has been done)
/// and wrap any scalar members in an object type.
fn wrap_scalar_union_variants(graph: &mut OpenApiGraph) {
    let unions = graph
        .graph
        .node_indices()
        .filter(|index| matches!(graph.graph[*index], Node::Union))
        .collect::<Vec<_>>();

    let mut wrapper_indices = HashMap::new();

    for union_index in unions {
        let scalar_members = graph
            .graph
            .edges(union_index)
            .filter_map(|edge| match edge.weight() {
                Edge::HasUnionMember => Some(edge.target()),
                _ => None,
            })
            .filter(|index| matches!(graph.graph[*index], Node::Scalar(_)))
            .collect::<Vec<_>>();

        for scalar_index in scalar_members {
            let scalar_kind = match &graph.graph[scalar_index] {
                Node::Scalar(kind) => *kind,
                _ => unreachable!(),
            };

            // Note: We don't remove the original edge because unions can appear in
            // both input _and_ output positions.
            // We need to keep the scalar values in place for input unions, only output
            // unions require these wrapper types.

            let wrapper_index = wrapper_indices
                .entry(scalar_kind)
                .or_insert_with(|| graph.graph.add_node(Node::UnionWrappedScalar(scalar_kind)));

            graph.graph.add_edge(union_index, *wrapper_index, Edge::HasUnionMember);
        }
    }
}

fn merge_all_of_schemas(graph: &mut OpenApiGraph) -> Result<(), Error> {
    let filtered_graph = petgraph::visit::NodeFiltered::from_fn(&graph.graph, |index| {
        matches!(&graph.graph[index], Node::Schema(_) | Node::AllOf)
    });

    let all_ofs = petgraph::algo::toposort(&filtered_graph, None)
        .map_err(|_| {
            // I think a cycle of AllOfs is probably against the spec - it wouldn't make sense...
            Error::AllOfCycle
        })?
        .into_iter()
        .rev()
        .filter(|index| matches!(graph.graph[*index], Node::AllOf))
        .collect::<Vec<_>>();

    for all_of_index in all_ofs {
        let members = resolve_all_of_members(graph, all_of_index);

        // We create an object to represent this allOf, then copy all the nested
        // fields onto that object.
        let object_index = graph.graph.add_node(Node::Object);
        for member in members {
            let mut new_edges = Vec::new();
            for edge in graph.graph.edges(member.index()) {
                if let Edge::HasField { name, wrapping } = edge.weight() {
                    new_edges.push((name.clone(), wrapping.clone(), edge.target()));
                }
            }

            for (name, wrapping, target_index) in new_edges {
                graph
                    .graph
                    .add_edge(object_index, target_index, Edge::HasField { name, wrapping });
            }
        }

        // Now we need to rewrite any edges to the allOf to point at our new object instead
        let mut new_edges = Vec::new();
        let mut edges_to_delete = Vec::new();
        for edge in graph.graph.edges_directed(all_of_index, Direction::Incoming) {
            new_edges.push((edge.source(), (*edge.weight()).clone()));
            edges_to_delete.push(edge.id());
        }
        for (source_index, weight) in new_edges {
            graph.graph.add_edge(source_index, object_index, weight);
        }
        for edge_index in edges_to_delete {
            graph.graph.remove_edge(edge_index);
        }
    }

    Ok(())
}

fn resolve_all_of_members(graph: &OpenApiGraph, all_of_index: NodeIndex) -> Vec<AllOfMember> {
    graph
        .graph
        .edges(all_of_index)
        .filter_map(|edge| matches!(edge.weight(), Edge::AllOfMember).then_some(edge.target()))
        .filter_map(|target_index| AllOfMember::from_index(target_index, graph))
        .collect()
}
