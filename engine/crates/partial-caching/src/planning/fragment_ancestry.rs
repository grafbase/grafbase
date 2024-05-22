use cynic_parser::executable::ids::{FragmentDefinitionId, SelectionId};
use indexmap::{IndexMap, IndexSet};

use super::{
    fragment_graph::{AncestorEdge, FragmentGraph},
    FragmentSpreadSet,
};

/// The complete ancestry of a particular fragment
///
/// We use FragmentGraph to build one of these for each fragment.
#[derive(Default)]
pub struct FragmentAncestry {
    /// All the fragments that contain spreads this fragment, directly or indirectly.
    ///
    /// These all need to be included in a query if this fragment is
    pub(super) fragments: IndexSet<FragmentDefinitionId>,

    /// All the selections that contain spreads of this fragment, directly or indirectly.
    ///
    /// These all need to be included in a query if this fragment is
    pub(super) selections: IndexSet<SelectionId>,
}

pub fn calculate_ancestry(
    fragments_in_fragments: IndexMap<FragmentDefinitionId, FragmentSpreadSet>,
    fragments_in_query: FragmentSpreadSet,
) -> IndexMap<FragmentDefinitionId, FragmentAncestry> {
    let graph = FragmentGraph::new(&fragments_in_fragments, &fragments_in_query);

    let mut ancestor_map = IndexMap::<FragmentDefinitionId, FragmentAncestry>::new();

    for fragment in graph.fragments() {
        let mut ancestry = FragmentAncestry::default();

        for edge in fragment.ancestor_edges() {
            let AncestorEdge { parent_id, child_id } = edge;

            match parent_id {
                Some(parent_id) => {
                    ancestry.fragments.insert(parent_id);

                    if let Some(parent_selections) = fragments_in_fragments
                        .get(&parent_id)
                        .and_then(|spread_set| spread_set.spreads_for_fragment(child_id))
                    {
                        ancestry.selections.extend(parent_selections);
                    }
                }
                None => {
                    // No parent indicates this is an edge to the query
                    if let Some(selections) = fragments_in_query.spreads_for_fragment(child_id) {
                        ancestry.selections.extend(selections);
                    }
                }
            }
        }

        ancestor_map.insert(fragment.id, ancestry);
    }
    ancestor_map
}
