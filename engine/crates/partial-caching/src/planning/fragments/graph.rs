use std::collections::HashSet;

use cynic_parser::executable::ids::FragmentDefinitionId;
use indexmap::{IndexMap, IndexSet};

use super::FragmentSpreadSet;

/// A graph of dependencies between fragments, used to calculate which fields/fragments
/// need to be included if a particular fragment is included.
pub struct FragmentGraph {
    /// All fragments in the query, and a set of fragments that contain that fragment.
    /// None in the set indicates this fragment is contained in the query
    direct_parents: IndexMap<FragmentDefinitionId, IndexSet<Option<FragmentDefinitionId>>>,
}

impl FragmentGraph {
    pub fn new(
        fragments_in_fragments: &IndexMap<FragmentDefinitionId, FragmentSpreadSet>,
        fragments_in_query: &FragmentSpreadSet,
    ) -> Self {
        let mut this = FragmentGraph {
            direct_parents: IndexMap::new(),
        };

        // Invert fragments_in_fragments so we have a map from child -> parents
        for (parent_id, children) in fragments_in_fragments {
            for child_id in children.fragment_ids() {
                this.direct_parents
                    .entry(child_id)
                    .or_default()
                    .insert(Some(*parent_id));
            }
        }

        // Add in the nodes that point to our query
        for child_id in fragments_in_query.fragment_ids() {
            this.direct_parents.entry(child_id).or_default().insert(None);
        }

        this
    }

    pub fn fragments(&self) -> impl Iterator<Item = Fragment<'_>> {
        self.direct_parents.keys().map(|id| Fragment { graph: self, id: *id })
    }
}

pub struct Fragment<'a> {
    graph: &'a FragmentGraph,
    pub id: FragmentDefinitionId,
}

impl<'a> Fragment<'a> {
    pub fn ancestor_edges(&self) -> AncestorEdgeIterator<'a> {
        let stack = self.graph.direct_parents[&self.id]
            .iter()
            .map(|parent_id| AncestorEdge {
                child_id: self.id,
                parent_id: *parent_id,
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
    edges_seen: HashSet<AncestorEdge>,
    stack: Vec<AncestorEdge>,
}

impl Iterator for AncestorEdgeIterator<'_> {
    type Item = AncestorEdge;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let edge = self.stack.pop()?;

            if self.edges_seen.contains(&edge) {
                // Fragment cycles aren't allowed but we check for them here incase
                // someone tries it.
                continue;
            }
            self.edges_seen.insert(edge);

            if let Some(parent) = edge.parent_id {
                if let Some(grandparents) = self.graph.direct_parents.get(&parent) {
                    self.stack
                        .extend(grandparents.iter().map(|grandparent_id| AncestorEdge {
                            child_id: parent,
                            parent_id: *grandparent_id,
                        }));
                }
            }

            return Some(edge);
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct AncestorEdge {
    pub parent_id: Option<FragmentDefinitionId>,
    pub child_id: FragmentDefinitionId,
}
