use std::collections::HashSet;

use indexmap::{IndexMap, IndexSet};

use super::{FragmentKey, FragmentSpreadSet};

/// A graph of dependencies between fragments, used to calculate which fields/fragments
/// need to be included if a particular fragment is included.
#[derive(Debug)]
pub struct FragmentGraph {
    /// All fragments in the query, and a set of fragments that contain that fragment.
    /// None in the set indicates this fragment is contained in the query
    direct_parents: IndexMap<FragmentKey, IndexSet<Option<FragmentKey>>>,
}

impl FragmentGraph {
    pub fn new(
        fragments_in_fragments: &IndexMap<FragmentKey, FragmentSpreadSet>,
        fragments_in_query: &FragmentSpreadSet,
    ) -> Self {
        let mut this = FragmentGraph {
            direct_parents: IndexMap::new(),
        };

        // Invert fragments_in_fragments so we have a map from child -> parents
        for (parent_key, children) in fragments_in_fragments {
            for child_key in children.fragment_keys() {
                this.direct_parents
                    .entry(child_key)
                    .or_default()
                    .insert(Some(parent_key.clone()));
            }
        }

        // Add in the nodes that point to our query
        for child_key in fragments_in_query.fragment_keys() {
            this.direct_parents.entry(child_key).or_default().insert(None);
        }

        this
    }

    pub fn fragments(&self) -> impl Iterator<Item = Fragment<'_>> {
        self.direct_parents.keys().map(|key| Fragment { graph: self, key })
    }
}

pub struct Fragment<'a> {
    graph: &'a FragmentGraph,
    pub key: &'a FragmentKey,
}

impl<'a> Fragment<'a> {
    pub fn ancestor_edges(&self) -> AncestorEdgeIterator<'a> {
        let stack = self.graph.direct_parents[self.key]
            .iter()
            .map(|parent_key| AncestorEdge {
                child_key: self.key,
                parent_key: parent_key.as_ref(),
            })
            .collect::<Vec<_>>();

        AncestorEdgeIterator {
            graph: self.graph,
            edges_seen: HashSet::new(),
            stack,
        }
    }
}

pub struct AncestorEdgeIterator<'a> {
    graph: &'a FragmentGraph,
    edges_seen: HashSet<AncestorEdge<'a>>,
    stack: Vec<AncestorEdge<'a>>,
}

impl<'a> Iterator for AncestorEdgeIterator<'a> {
    type Item = AncestorEdge<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let edge = self.stack.pop()?;

            if self.edges_seen.contains(&edge) {
                // Fragment cycles aren't allowed but we check for them here incase
                // someone tries it.
                continue;
            }
            self.edges_seen.insert(edge);

            if let Some(parent) = edge.parent_key {
                if let Some(grandparents) = self.graph.direct_parents.get(parent) {
                    self.stack
                        .extend(grandparents.iter().map(|grandparent_id| AncestorEdge {
                            child_key: parent,
                            parent_key: grandparent_id.as_ref(),
                        }));
                }
            }

            return Some(edge);
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct AncestorEdge<'a> {
    pub parent_key: Option<&'a FragmentKey>,
    pub child_key: &'a FragmentKey,
}
