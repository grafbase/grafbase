//! We can figure out a lot of the transformations between our OpenAPI and the eventual
//! schema in code as part of the typed layer we build on top of the graph.
//!
//! But some of these are quite tricky to implement there so we apply them directly
//! to the graph prior to building the typed layer.

use std::collections::{BTreeMap, HashMap};

use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

use super::{all_of_member::AllOfMember, Edge, Node, OpenApiGraph, WrappingType};
use crate::Error;

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

            let json_node = graph.graph.add_node(Node::Scalar(super::ScalarKind::Json));

            for type_edge in type_edges {
                let (source_node, _) = graph.graph.edge_endpoints(type_edge).unwrap();
                match graph.graph.remove_edge(type_edge).unwrap() {
                    Edge::HasType { wrapping } => {
                        graph.graph.add_edge(source_node, json_node, Edge::HasType { wrapping });
                    }
                    Edge::HasField {
                        name,
                        wrapping,
                        required,
                    } => {
                        graph.graph.add_edge(
                            source_node,
                            json_node,
                            Edge::HasField {
                                name,
                                wrapping,
                                required,
                            },
                        );
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

        let mut fields = BTreeMap::<String, AllOfField>::new();

        // Iterate through all the members grabbing fields merging the details of any duplicates.
        for member in members {
            for edge in graph.graph.edges(member.index()) {
                if let Edge::HasField {
                    name,
                    wrapping,
                    required,
                } = edge.weight()
                {
                    let field = AllOfField {
                        name: name.clone(),
                        required: *required,
                        wrapping: wrapping.clone(),
                        target_index: edge.target(),
                    };

                    fields
                        .entry(name.clone())
                        .and_modify(|existing_field| existing_field.merge(&field, graph))
                        .or_insert(field);
                }
            }
        }

        let object_index = graph.graph.add_node(Node::Object);
        for AllOfField {
            name,
            wrapping,
            required,
            target_index,
        } in fields.into_values()
        {
            graph.graph.add_edge(
                object_index,
                target_index,
                Edge::HasField {
                    name,
                    required,
                    wrapping,
                },
            );
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

#[derive(Debug, PartialEq)]
struct AllOfField {
    name: String,
    required: bool,
    wrapping: WrappingType,
    target_index: NodeIndex,
}

impl AllOfField {
    fn merge(&mut self, other: &AllOfField, graph: &OpenApiGraph) {
        assert_eq!(other.name, self.name);
        match (&graph.graph[self.target_index], &graph.graph[other.target_index]) {
            (Node::PlaceholderType, _) => {
                self.target_index = other.target_index;
                self.wrapping = other.wrapping.clone();
                self.required |= other.required;
            }
            (_, Node::PlaceholderType) => {
                self.required |= other.required;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use petgraph::Graph;

    use super::*;

    #[test]
    fn test_all_of_merge_lhs() {
        let mut graph = Graph::new();
        let boolean_type = graph.add_node(Node::Scalar(crate::graph::ScalarKind::Boolean));
        let placeholder_type = graph.add_node(Node::PlaceholderType);
        let graph = OpenApiGraph::from_petgraph(graph);

        let mut required_placeholder = AllOfField {
            name: "field".into(),
            wrapping: WrappingType::Named,
            required: true,
            target_index: placeholder_type,
        };
        let nullable_list_of_required_bools = AllOfField {
            name: "field".into(),
            wrapping: WrappingType::Named.wrap_required().wrap_list(),
            required: false,
            target_index: boolean_type,
        };

        required_placeholder.merge(&nullable_list_of_required_bools, &graph);

        assert_eq!(
            required_placeholder,
            AllOfField {
                required: true,
                ..nullable_list_of_required_bools
            }
        );
    }

    #[test]
    fn test_all_of_merge_rhs() {
        let mut graph = Graph::new();
        let boolean_type = graph.add_node(Node::Scalar(crate::graph::ScalarKind::Boolean));
        let placeholder_type = graph.add_node(Node::PlaceholderType);
        let graph = OpenApiGraph::from_petgraph(graph);

        let required_placeholder = AllOfField {
            name: "field".into(),
            wrapping: WrappingType::Named,
            required: true,
            target_index: placeholder_type,
        };
        let mut nullable_list_of_required_bools = AllOfField {
            name: "field".into(),
            wrapping: WrappingType::Named.wrap_required().wrap_list(),
            required: false,
            target_index: boolean_type,
        };

        nullable_list_of_required_bools.merge(&required_placeholder, &graph);

        assert_eq!(
            nullable_list_of_required_bools,
            AllOfField {
                name: "field".into(),
                wrapping: WrappingType::Named.wrap_required().wrap_list(),
                required: true,
                target_index: boolean_type,
            }
        );
    }

    // TODO: Do I also want a more thorough test of everything?  Not sure...
}
