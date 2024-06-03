use cynic_parser::executable::ids::{FragmentDefinitionId, SelectionId};
use indexmap::{IndexMap, IndexSet};

use super::{
    graph::{AncestorEdge, FragmentGraph},
    FragmentKey, FragmentSpreadSet,
};

/// The complete ancestry of a particular fragment
///
/// We use FragmentGraph to build one of these for each fragment.
#[derive(Default)]
pub struct FragmentAncestry {
    /// All the fragments that contain spreads this fragment, directly or indirectly.
    ///
    /// These all need to be included in a query if this fragment is
    pub(crate) fragments: IndexSet<FragmentDefinitionId>,

    /// All the selections that contain spreads of this fragment, directly or indirectly.
    ///
    /// These all need to be included in a query if this fragment is
    pub(crate) selections: IndexSet<SelectionId>,
}

pub fn calculate_ancestry(
    fragments_in_fragments: IndexMap<FragmentKey, FragmentSpreadSet>,
    fragments_in_query: FragmentSpreadSet,
) -> IndexMap<FragmentKey, FragmentAncestry> {
    let graph = FragmentGraph::new(&fragments_in_fragments, &fragments_in_query);

    let mut ancestor_map = IndexMap::<FragmentKey, FragmentAncestry>::new();

    for fragment in graph.fragments() {
        let mut ancestry = FragmentAncestry::default();

        for edge in fragment.ancestor_edges() {
            let AncestorEdge { parent_key, child_key } = edge;

            match parent_key {
                Some(parent_key) => {
                    ancestry.fragments.insert(parent_key.id);

                    if let Some(parent_selections) = fragments_in_fragments
                        .get(parent_key)
                        .and_then(|spread_set| spread_set.spreads_for_fragment(child_key))
                    {
                        ancestry.selections.extend(parent_selections);
                    }
                }
                None => {
                    // No parent indicates this is an edge to the query
                    if let Some(selections) = fragments_in_query.spreads_for_fragment(child_key) {
                        ancestry.selections.extend(selections);
                    }
                }
            }
        }

        ancestor_map.insert(fragment.key.clone(), ancestry);
    }
    ancestor_map
}
