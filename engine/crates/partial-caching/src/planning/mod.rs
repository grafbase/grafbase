mod cache_grouper;
mod fragment_graph;
mod fragment_tracker;
mod visitor;

use cynic_parser::{
    common::OperationType,
    executable::ids::{FragmentDefinitionId, SelectionId},
    ExecutableDocument,
};
use indexmap::{IndexMap, IndexSet};
use registry_for_cache::PartialCacheRegistry;

use self::{
    cache_grouper::QueryPartitioner,
    fragment_graph::{AncestorEdge, FragmentGraph},
    fragment_tracker::FragmentTracker,
    visitor::{visit_fragment, visit_query, VisitorContext},
};
use crate::{
    query_subset::{CacheGroup, QuerySubset},
    CachingPlan,
};

pub fn build_plan(
    query: &str,
    operation_name: Option<&str>,
    registry: &PartialCacheRegistry,
) -> anyhow::Result<Option<CachingPlan>> {
    let document = cynic_parser::parse_executable_document(query)?;

    let operation = match operation_name {
        Some(operation_name) => document
            .operations()
            .find(|op| op.name() == Some(operation_name))
            .ok_or_else(|| anyhow::anyhow!("Could not find an operation named {operation_name}"))?,
        None => document
            .operations()
            .next()
            .ok_or_else(|| anyhow::anyhow!("the graphql document contains no operations"))?,
    };

    if operation.operation_type() != OperationType::Query {
        // We don't cache mutations or subscriptions
        return Ok(None);
    }

    let mut partitioner = QueryPartitioner::new();
    let mut fragment_tracker = FragmentTracker::new();

    visit_query(
        operation,
        registry,
        &mut VisitorContext::new(&mut [&mut partitioner, &mut fragment_tracker]),
    );

    let (cache_groups, uncached_group) = visit_fragments(&document, registry, fragment_tracker, partitioner)?;

    let operation = operation.id();

    // TODO: probably want to return none if there are no cache groups
    // and its all in executor query...

    Ok(Some(CachingPlan {
        cache_queries: cache_groups
            .into_iter()
            .map(|(control, group)| (control, QuerySubset::new(operation, group, &document)))
            .collect(),
        executor_query: QuerySubset::new(operation, uncached_group, &document),
        document,
    }))
}

// TODO: This name is awful
#[derive(Default)]
struct FragmentChildren {
    /// Fragments selected in an operation or fragment, and the
    /// selections that need to be included from that operation or fragment
    /// if the nested fragment needs to be included
    fragments_selected: IndexMap<FragmentDefinitionId, IndexSet<SelectionId>>,
}

impl FragmentChildren {
    fn from_tracker(tracker: FragmentTracker, document: &ExecutableDocument) -> anyhow::Result<Self> {
        let mut this = Self::default();
        for (fragment_name, selections) in tracker.used_fragments {
            let fragment = document
                .fragments()
                .find(|fragment| fragment.name() == fragment_name)
                .ok_or_else(|| {
                    anyhow::anyhow!("The query contained a spread for a missing fragment: {fragment_name}")
                })?;

            this.fragments_selected.insert(fragment.id(), selections);
        }

        Ok(this)
    }

    fn fragment_ids(&self) -> impl Iterator<Item = FragmentDefinitionId> + '_ {
        self.fragments_selected.keys().copied()
    }
}

fn visit_fragments(
    document: &cynic_parser::ExecutableDocument,
    registry: &PartialCacheRegistry,
    fragment_tracker: FragmentTracker,
    mut partitioner: QueryPartitioner,
) -> anyhow::Result<(
    IndexMap<registry_for_cache::CacheControl, crate::query_subset::CacheGroup>,
    crate::query_subset::CacheGroup,
)> {
    let query_fragment_children = FragmentChildren::from_tracker(fragment_tracker, document)?;

    let mut fragments_to_visit = query_fragment_children.fragment_ids().collect::<Vec<_>>();
    let mut fragment_child_map = IndexMap::<FragmentDefinitionId, FragmentChildren>::new();

    while let Some(fragment_id) = fragments_to_visit.pop() {
        let fragment = document.read(fragment_id);

        if fragment_child_map.contains_key(&fragment_id) {
            continue;
        }

        partitioner = partitioner.for_next_fragment(fragment_id);

        let mut fragment_tracker = FragmentTracker::new();
        visit_fragment(
            fragment,
            registry,
            &mut VisitorContext::new(&mut [&mut partitioner, &mut fragment_tracker]),
        );

        let fragment_children = FragmentChildren::from_tracker(fragment_tracker, document)?;
        fragments_to_visit.extend(fragment_children.fragment_ids());
        fragment_child_map.insert(fragment_id, fragment_children);
    }

    let ancestor_map = build_ancestor_map(fragment_child_map, query_fragment_children);

    let mut cache_partitions = partitioner.cache_partitions;
    let mut nocache_partition = partitioner.nocache_partition;

    nocache_partition.update_with_fragment_ancestors(&ancestor_map);
    for group in cache_partitions.values_mut() {
        group.update_with_fragment_ancestors(&ancestor_map);
    }

    Ok((cache_partitions, nocache_partition))
}

/// The ancestry of a particular fragment
#[derive(Default)]
struct FragmentAncestry {
    /// All the fragments that contain spreads this fragment, directly or indirectly.
    ///
    /// These all need to be included in a query if this fragment is
    fragments: IndexSet<FragmentDefinitionId>,

    /// All the selections that contain spreads of this fragment, directly or indirectly.
    ///
    /// These all need to be included in a query if this fragment is
    selections: IndexSet<SelectionId>,
}

fn build_ancestor_map(
    fragment_child_map: IndexMap<FragmentDefinitionId, FragmentChildren>,
    query_fragment_children: FragmentChildren,
) -> IndexMap<FragmentDefinitionId, FragmentAncestry> {
    let graph = FragmentGraph::new(&fragment_child_map, &query_fragment_children);

    let mut ancestor_map = IndexMap::<FragmentDefinitionId, FragmentAncestry>::new();

    for fragment in graph.fragments() {
        let mut ancestry = FragmentAncestry::default();

        for edge in fragment.ancestor_edges() {
            let AncestorEdge { parent_id, child_id } = edge;

            match parent_id {
                Some(parent_id) => {
                    ancestry.fragments.insert(parent_id);

                    if let Some(parent_selections) = fragment_child_map
                        .get(&parent_id)
                        .and_then(|parent| parent.fragments_selected.get(&child_id))
                    {
                        ancestry.selections.extend(parent_selections);
                    }
                }
                None => {
                    // No parent indicates this is an edge to the query, so look up in query_fragment_children
                    if let Some(root_selections) = query_fragment_children.fragments_selected.get(&child_id) {
                        ancestry.selections.extend(root_selections);
                    }
                }
            }
        }

        ancestor_map.insert(fragment.id, ancestry);
    }
    ancestor_map
}

impl CacheGroup {
    fn update_with_fragment_ancestors(&mut self, ancestors: &IndexMap<FragmentDefinitionId, FragmentAncestry>) {
        for fragment_id in self.fragments.iter().copied().collect::<Vec<_>>() {
            if let Some(ancestors) = ancestors.get(&fragment_id) {
                self.selections.extend(ancestors.selections.iter().copied());
                self.fragments.extend(ancestors.fragments.iter().copied())
            }
        }
    }
}
