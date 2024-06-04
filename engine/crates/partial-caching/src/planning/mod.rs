mod fragments;
mod query_partitioner;
mod variables;
mod visitor;

use cynic_parser::common::OperationType;
use fragments::FragmentKey;
use indexmap::{IndexMap, IndexSet};
use query_partitioner::PlanningPartition;
use registry_for_cache::PartialCacheRegistry;
use variables::variables_required;

use self::{
    fragments::{calculate_ancestry, FragmentAncestry, FragmentSpreadSet, FragmentTracker},
    query_partitioner::QueryPartitioner,
    visitor::{visit_fragment, visit_query, VisitorContext},
};
use crate::{
    query_subset::{Partition, QuerySubset},
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

    let root_cache_control = registry.query_type().cache_control();
    let mut partitioner = QueryPartitioner::new(root_cache_control);
    let mut fragment_tracker = FragmentTracker::new(root_cache_control);

    visit_query(
        operation,
        registry,
        &mut VisitorContext::new(&mut [&mut partitioner, &mut fragment_tracker]),
    );

    let (cache_groups, uncached_group) = visit_fragments(&document, registry, fragment_tracker, partitioner)?;

    let nocache_variables = variables_required(&uncached_group, &document, operation);

    Ok(Some(CachingPlan {
        cache_partitions: cache_groups
            .into_iter()
            .map(|(control, group)| {
                let variables = variables_required(&group, &document, operation);
                (control, QuerySubset::new(operation.id(), group, variables))
            })
            .collect(),
        nocache_partition: QuerySubset::new(operation.id(), uncached_group, nocache_variables),
        document,
    }))
}

fn visit_fragments(
    document: &cynic_parser::ExecutableDocument,
    registry: &PartialCacheRegistry,
    fragment_tracker: FragmentTracker,
    mut partitioner: QueryPartitioner,
) -> anyhow::Result<(
    IndexMap<registry_for_cache::CacheControl, crate::query_subset::Partition>,
    crate::query_subset::Partition,
)> {
    let fragments_in_query = fragment_tracker.into_spreads()?;

    let mut fragments_to_visit = fragments_in_query.fragment_keys().collect::<Vec<_>>();
    let mut fragments_in_fragments = IndexMap::<FragmentKey, FragmentSpreadSet>::new();

    while let Some(fragment_key) = fragments_to_visit.pop() {
        let fragment = document.read(fragment_key.id);
        if fragments_in_fragments.contains_key(&fragment_key) {
            continue;
        }

        partitioner = partitioner.for_next_fragment(fragment_key.clone());
        let mut fragment_tracker = FragmentTracker::new(fragment_key.spread_cache_control.as_ref());

        visit_fragment(
            fragment,
            registry,
            &mut VisitorContext::new(&mut [&mut partitioner, &mut fragment_tracker]),
        );

        let spreads = fragment_tracker.into_spreads()?;

        fragments_to_visit.extend(spreads.fragment_keys());

        fragments_in_fragments.insert(fragment_key, spreads);
    }

    let ancestor_map = calculate_ancestry(fragments_in_fragments, fragments_in_query);

    let nocache_partition = partitioner.nocache_partition.finalize(&ancestor_map);

    let cache_partitions = partitioner
        .cache_partitions
        .into_iter()
        .map(|(cache_control, group)| (cache_control, group.finalize(&ancestor_map)))
        .collect();

    Ok((cache_partitions, nocache_partition))
}

impl PlanningPartition {
    fn finalize(self, ancestors: &IndexMap<FragmentKey, FragmentAncestry>) -> Partition {
        let PlanningPartition { selections, fragments } = self;
        let mut partition = Partition {
            selections,
            fragments: IndexSet::new(),
        };

        for key in fragments {
            partition.fragments.insert(key.id);
            if let Some(ancestors) = ancestors.get(&key) {
                partition.selections.extend(ancestors.selections.iter().copied());
                partition.fragments.extend(ancestors.fragments.iter().copied())
            }
        }

        partition
    }
}

impl<'a> visitor::FieldEdge<'a> {
    fn cache_control(&self) -> Option<&'a registry_for_cache::CacheControl> {
        let field_cache_control = self.field.and_then(|field| field.cache_control());
        let type_cache_control = self.field_type.and_then(|ty| ty.cache_control());

        field_cache_control.or(type_cache_control)
    }
}
