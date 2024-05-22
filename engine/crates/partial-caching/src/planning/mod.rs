mod fragment_ancestry;
mod fragment_graph;
mod fragment_tracker;
mod query_partitioner;
mod selected_fragments;
mod visitor;

use cynic_parser::{common::OperationType, executable::ids::FragmentDefinitionId};
use indexmap::IndexMap;
use registry_for_cache::PartialCacheRegistry;

use self::{
    fragment_ancestry::{calculate_ancestry, FragmentAncestry},
    fragment_tracker::FragmentTracker,
    query_partitioner::QueryPartitioner,
    selected_fragments::FragmentSpreadSet,
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

fn visit_fragments(
    document: &cynic_parser::ExecutableDocument,
    registry: &PartialCacheRegistry,
    fragment_tracker: FragmentTracker,
    mut partitioner: QueryPartitioner,
) -> anyhow::Result<(
    IndexMap<registry_for_cache::CacheControl, crate::query_subset::CacheGroup>,
    crate::query_subset::CacheGroup,
)> {
    let fragments_in_query = FragmentSpreadSet::from_tracker(fragment_tracker, document)?;

    let mut fragments_to_visit = fragments_in_query.fragment_ids().collect::<Vec<_>>();
    let mut fragments_in_fragments = IndexMap::<FragmentDefinitionId, FragmentSpreadSet>::new();

    while let Some(fragment_id) = fragments_to_visit.pop() {
        let fragment = document.read(fragment_id);
        if fragments_in_fragments.contains_key(&fragment_id) {
            continue;
        }

        partitioner = partitioner.for_next_fragment(fragment_id);
        let mut fragment_tracker = FragmentTracker::new();

        visit_fragment(
            fragment,
            registry,
            &mut VisitorContext::new(&mut [&mut partitioner, &mut fragment_tracker]),
        );

        let spreads = FragmentSpreadSet::from_tracker(fragment_tracker, document)?;

        fragments_to_visit.extend(spreads.fragment_ids());

        fragments_in_fragments.insert(fragment_id, spreads);
    }

    let ancestor_map = calculate_ancestry(fragments_in_fragments, fragments_in_query);

    let mut cache_partitions = partitioner.cache_partitions;
    let mut nocache_partition = partitioner.nocache_partition;

    nocache_partition.update_with_fragment_ancestors(&ancestor_map);
    for group in cache_partitions.values_mut() {
        group.update_with_fragment_ancestors(&ancestor_map);
    }

    Ok((cache_partitions, nocache_partition))
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
