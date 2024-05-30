//! Implements the execution phase of caching - quite a simple one this, it just
//! takes the original query, removes any parts for which we have cache it and
//! provides whatever is left.  This can be passed to the executor to run the
//! query.

use cynic_parser::ExecutableDocument;
use graph_entities::{QueryResponse, ResponseNodeId};
use registry_for_cache::CacheControl;
use runtime::cache::Entry;

use crate::QuerySubset;

use super::fetching::CacheFetchPhase;

#[allow(unused)] // Going to update things to use this later
pub struct ExecutionPhase {
    document: ExecutableDocument,
    cache_partitions: Vec<(CacheControl, QuerySubset)>,
    cache_entries: Vec<Entry<serde_json::Value>>,
    cache_keys: Vec<Option<String>>,
    executor_subset: QuerySubset,
}

impl ExecutionPhase {
    pub(crate) fn new(fetch_phase: CacheFetchPhase) -> Self {
        let plan = fetch_phase.plan;

        let mut executor_subset = plan.nocache_partition;
        for (entry, (_, partition_subset)) in fetch_phase.cache_entries.iter().zip(plan.cache_partitions.iter()) {
            if entry.is_miss() {
                executor_subset.extend(partition_subset);
            }
        }

        Self {
            document: plan.document,
            cache_partitions: plan.cache_partitions,
            cache_keys: fetch_phase.cache_keys,
            cache_entries: fetch_phase.cache_entries,
            executor_subset,
        }
    }

    pub fn query(&self) -> String {
        self.executor_subset
            .as_display(&self.document)
            .include_query_name()
            .to_string()
    }

    pub fn handle_response(self, mut response: QueryResponse) -> (QueryResponse, CacheUpdates) {
        for entry in self.cache_entries {
            match entry {
                Entry::Hit(hit) => merge_json(&mut response, hit),
                Entry::Stale(stale) => {
                    // TODO: Also want to issue an update instruction here, but going to do that
                    // in GB-6804
                    merge_json(&mut response, stale.value)
                }
                Entry::Miss => {}
            }
        }
        (response, CacheUpdates)
    }
}

// TODO: Going to do this in next PR.
pub struct CacheUpdates;

pub(super) fn merge_json(response: &mut QueryResponse, json: serde_json::Value) {
    use graph_entities::QueryResponseNode;
    use serde_json::Value;

    let Some(current_node_id) = response.root else {
        // Presumably an error bubbled up to the root, so not much we can do here.
        return;
    };

    fn inner(response: &mut QueryResponse, json: Value, current_node_id: ResponseNodeId) {
        let Some(node) = response.get_node(current_node_id) else {
            unreachable!("every node ID should exist")
        };

        match (node, json) {
            (QueryResponseNode::Container(_), Value::Object(cache_fields)) => {
                for (field, field_value) in cache_fields {
                    match response.get_container_node(current_node_id).unwrap().child(&field) {
                        Some(child_id) => inner(response, field_value, child_id),
                        None => {
                            let field_id = response.from_serde_value(field_value);
                            let mutable_container = response.get_container_node_mut(current_node_id).unwrap();
                            mutable_container.insert(&field, field_id);
                        }
                    }
                }
            }
            (QueryResponseNode::Container(_), _) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
            (QueryResponseNode::List(response_list), Value::Array(cache_list)) => {
                // Note: for now we are being very very naive and assuming the list order & length always matches
                // I have a linear task (GB-6782) to make this less naive
                let response_items = response_list.iter().collect::<Vec<_>>();
                for (response_item, cache_item) in response_items.into_iter().zip(cache_list) {
                    inner(response, cache_item, response_item)
                }
            }
            (QueryResponseNode::List(_), _) => todo!("this is a problem"),
            (QueryResponseNode::Primitive(primitive), _) if primitive.is_null() => {
                // This quite probably means that an error occurred in the execution and
                // it's bubbled up to some other field.  We can't merge any cache results nested beneath
                // this so just do nothing.
            }
            (QueryResponseNode::Primitive(_), _) => {
                // TODO: Going to deal with this in GB-6782
                todo!("probably need to invalidate cache if this happens");
            }
        }
    }

    inner(response, json, current_node_id)
}
