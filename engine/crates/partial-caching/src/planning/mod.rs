mod cache_grouper;
mod fragment_tracker;
mod visitor;

use std::collections::HashSet;

use cynic_parser::{
    common::OperationType,
    executable::ids::{FragmentDefinitionId, SelectionId},
    ExecutableDocument,
};
use indexmap::{IndexMap, IndexSet};
use registry_for_cache::PartialCacheRegistry;

use self::{
    cache_grouper::CacheGrouper,
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

    let mut cache_group_visitor = CacheGrouper::new();
    let mut fragment_tracker = FragmentTracker::new();

    visit_query(
        operation,
        registry,
        // TODO: This API really needs a rethink
        &mut VisitorContext::new(&mut [&mut cache_group_visitor, &mut fragment_tracker]),
    );

    let (cache_groups, uncached_group) = visit_fragments(&document, registry, fragment_tracker, cache_group_visitor)?;

    let operation = operation.id();

    Ok(Some(CachingPlan {
        cache_queries: cache_groups
            .into_iter()
            .map(|(control, group)| (control, QuerySubset::new(operation, group, &document)))
            .collect(),
        executor_query: QuerySubset::new(operation, uncached_group, &document),
        document,
    }))

    // TODO:
    // 1. Parse the query
    // 2. Walk the query in parallel w/ the registry.
    // 3. At leaf field, calculate the current cache setting
    // 4. Add all nodes in heirarchy to a group for the current cache setting.
    // 5. Also need to record which variables & fragments are used by each leaf/group/whatever.
    //    Either recorded as we go or as a post-processing step.
    //
    //    Recorded as we go for variables means:
    //    - At the point of recording a field into a group we need to both look at all the parents and gather their varaibles,
    //      and then gather any variables used by the current field.
    //
    //    Post-processing for variables means 1 additional walk of the processed struct (which seems less work overall)
    //
    //    Fragments we probably do want to track as we go.
    //
    // 6. Need to traverse any fragments as well...
    //    Assuming we're not doing any fancy traversal based cache rules then
    //    this is simple: just track any fragments used by the query and traverse them all
    //    individually, adding their field sets to the set of OK things.
    //
    // 7. Printing any individual query doc then just becomes a case of traversing the
    //    "raw" graph but with some extra "is this node in the current set" filtering.
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
    mut cache_group_visitor: CacheGrouper,
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

        cache_group_visitor = cache_group_visitor.with_current_fragment(fragment_id);

        let mut fragment_tracker = FragmentTracker::new();
        visit_fragment(
            fragment,
            registry,
            &mut VisitorContext::new(&mut [&mut cache_group_visitor, &mut fragment_tracker]),
        );

        let fragment_children = FragmentChildren::from_tracker(fragment_tracker, document)?;
        fragments_to_visit.extend(fragment_children.fragment_ids());
        fragment_child_map.insert(fragment_id, fragment_children);
    }

    let ancestor_map = build_ancestor_map(fragment_child_map, query_fragment_children);

    let mut cache_groups = cache_group_visitor.cache_groups;
    let mut uncached_group = cache_group_visitor.uncached_group;

    uncached_group.update_with_fragment_ancestors(&ancestor_map);
    for group in cache_groups.values_mut() {
        group.update_with_fragment_ancestors(&ancestor_map);
    }

    Ok((cache_groups, uncached_group))
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
    let mut direct_parents = IndexMap::<FragmentDefinitionId, IndexSet<FragmentDefinitionId>>::new();

    // Invert fragment_child_map so we have a map from child -> parents
    for (parent_id, children) in &fragment_child_map {
        for child_id in children.fragments_selected.keys() {
            direct_parents.entry(*child_id).or_default().insert(*parent_id);
        }
    }

    let mut ancestor_map = IndexMap::<FragmentDefinitionId, FragmentAncestry>::new();

    // Now that we have all the direct parents we can calculate ancestors
    let mut edges_seen = HashSet::new();
    for (child_id, parent_ids) in &direct_parents {
        // TODO: Probably extract the traversal logic into an iterator
        let entry = ancestor_map.entry(*child_id).or_default();
        let mut edge_stack = parent_ids
            .into_iter()
            .map(|parent_id| (child_id, parent_id))
            .collect::<Vec<_>>();

        edges_seen.clear();
        while let Some(edge) = edge_stack.pop() {
            if edges_seen.contains(&edge) {
                // There shouldn't be cycles in fragments, but lets err on the safe side.
                continue;
            }
            edges_seen.insert(edge);

            let (child_id, parent_id) = edge;

            if let Some(grandparents) = direct_parents.get(parent_id) {
                edge_stack.extend(grandparents.iter().map(|grandparent_id| (parent_id, grandparent_id)));
            }

            entry.fragments.insert(*parent_id);

            if let Some(parent_selections) = fragment_child_map
                .get(parent_id)
                .and_then(|child| child.fragments_selected.get(child_id))
            {
                entry.selections.extend(parent_selections);
            }

            if let Some(root_selections) = query_fragment_children.fragments_selected.get(parent_id) {
                entry.selections.extend(root_selections);
            }
        }

        if let Some(root_selections) = query_fragment_children.fragments_selected.get(child_id) {
            entry.selections.extend(root_selections);
        }
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
