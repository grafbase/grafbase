use indexmap::{IndexMap, IndexSet};
use petgraph::{graphmap::GraphMap, visit::EdgeRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Node<'a> {
    Type(&'a str),
    Field(&'a str, &'a str),
}

type Graph<'a> = GraphMap<Node<'a>, (), petgraph::Directed>;

use crate::{MetaType, Registry};

impl super::Registry {
    /// Prunes the Registry to only types that we need for caching.
    ///
    /// This means any types that:
    /// - Have cache_control on them
    /// - Have any fields with cache_control on them
    /// - Are used on any fields with cache_control.
    /// - Have any descendants that have cache_control on them.
    pub fn prune_for_caching_registry(self) -> Self {
        let types = self.types;

        // This algorithm is basically just a big DFS on the type graph,
        // so lets use petgraph to save implementing it by hand.
        let graph = build_type_graph(&types);

        // Map where keys are types to retain, value is list of fields we should retain on that type
        let mut retain_map = dbg!(types_and_fields_marked_cache(&types));
        let specifically_cached_types = retain_map.keys().map(|key| Node::Type(key)).collect::<Vec<_>>();

        let graph = petgraph::visit::EdgeFiltered::from_fn(&graph, |edge| {
            // Stop the traversal when we hit the Query type.
            edge.source() != Node::Type(&self.query_type)
        });
        let mut dfs = petgraph::visit::Dfs::empty(&graph);
        dfs.stack.extend(specifically_cached_types);
        while let Some(node) = dfs.next(&graph) {
            match node {
                Node::Type(ty) => {
                    retain_map.entry(ty).or_default();

                    if let Some(MetaType::Interface(iface)) = types.get(ty) {
                        for ty in &iface.possible_types {
                            retain_map.entry(ty);
                        }
                    }
                }
                Node::Field(ty, field) => {
                    retain_map.entry(ty).or_default().push(field);
                }
            }
        }

        retain_field_targets(&mut retain_map, &types);

        let types_with_cache = types
            .iter()
            .filter_map(|(type_name, type_value)| {
                let keys_to_retain = retain_map.get(type_name.as_str())?;

                let type_name = type_name.clone();
                let mut type_value = type_value.clone();

                if let Some(fields) = type_value.fields_mut() {
                    fields.retain(|name, _| keys_to_retain.contains(&name.as_str()));
                }

                Some((type_name, type_value))
            })
            .collect();

        Registry {
            enable_caching: self.enable_caching,
            types: types_with_cache,
            ..Default::default()
        }
    }
}

fn build_type_graph(types: &std::collections::BTreeMap<String, MetaType>) -> Graph<'_> {
    // This won't be accurate, but its a starting point
    let mut graph = Graph::with_capacity(types.len(), types.len());

    for ty in types.values() {
        graph.add_node(Node::Type(ty.name()));
    }

    for ty in types.values() {
        let Some(fields) = ty.fields() else { continue };
        let container_type = Node::Type(ty.name());
        for field in fields.values() {
            let field_node = Node::Field(ty.name(), field.name.as_str());
            let field_type = Node::Type(field.ty.base_type_name());
            graph.add_node(field_node);

            // We're interested in walking the tree from cacheable fields/types to
            // the root, so we add our edges in that direction.
            graph.add_edge(field_type, field_node, ());
            graph.add_edge(field_node, container_type, ());
        }
    }

    graph
}

/// Finds the types and fields that need to be handled for caching
fn types_and_fields_marked_cache(types: &std::collections::BTreeMap<String, MetaType>) -> IndexMap<&str, Vec<&str>> {
    types
        .iter()
        .filter_map(|(type_name, type_value)| {
            // it is expected that the Query node is always present as it is the starting point
            // for validation visiting. check rules/visitor.rs:588
            if *type_name == "Query" {
                return Some((type_name.as_str(), vec![]));
            }

            match type_value {
                MetaType::Object(object) if object.cache_control.is_some() => {
                    let fields = object
                        .fields
                        .values()
                        .filter(|field| field.cache_control.is_some())
                        .map(|field| field.name.as_str())
                        .collect();

                    Some((type_name.as_str(), fields))
                }
                MetaType::Interface(interface) if interface.cache_control.is_some() => {
                    let fields = interface
                        .fields
                        .values()
                        .filter(|field| field.cache_control.is_some())
                        .map(|field| field.name.as_str())
                        .collect();

                    Some((type_name.as_str(), fields))
                }
                _ => None,
            }
        })
        .collect()
}

// At this point we have a map of all the types & fields specifically marked for caching.
// We need to go through any of those fields and make sure that their target types
// are also marked for retention
fn retain_field_targets<'a>(
    retain_map: &mut IndexMap<&'a str, Vec<&'a str>>,
    types: &'a std::collections::BTreeMap<String, MetaType>,
) {
    let mut retain_types = IndexSet::new();
    for (ty, fields) in retain_map.iter() {
        let meta_type = &types[*ty];
        for field_name in fields {
            let Some(fields) = meta_type.fields() else {
                continue;
            };
            retain_types.insert(fields[*field_name].ty.base_type_name());
        }
    }

    for ty in retain_types {
        retain_map.entry(ty).or_default();
    }
}
