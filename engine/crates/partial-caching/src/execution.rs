//! Implements the execution phase of caching - quite a simple one this, it just
//! takes the original query, removes any parts for which we have cache it and
//! provides whatever is left.  This can be passed to the executor to run the
//! query.

use std::time::Duration;

use cynic_parser::ExecutableDocument;
use graph_entities::{QueryResponse, ResponseNodeId};
use registry_for_cache::CacheControl;
use runtime::cache::Entry;

use crate::{
    headers::RequestCacheControl, response::MaxAge, updating::PartitionIndex, CacheUpdatePhase, QuerySubset, Response,
};

use super::fetching::CacheFetchPhase;

pub struct ExecutionPhase {
    document: ExecutableDocument,
    cache_partitions: Vec<(CacheControl, QuerySubset)>,
    cache_entries: Vec<Entry<serde_json::Value>>,
    cache_keys: Vec<Option<String>>,
    executor_subset: QuerySubset,
    cache_miss_count: usize,
    is_partial_hit: bool,
    has_nocache_partition: bool,

    request_cache_control: RequestCacheControl,
}

impl ExecutionPhase {
    pub(crate) fn new(fetch_phase: CacheFetchPhase) -> Self {
        let plan = fetch_phase.plan;
        let has_nocache_partition = !plan.nocache_partition.is_empty();

        let mut is_partial_hit = false;
        let mut cache_miss_count = 0;
        let mut executor_subset = plan.nocache_partition;
        for (entry, (_, partition_subset)) in fetch_phase.cache_entries.iter().zip(plan.cache_partitions.iter()) {
            if entry.is_miss() {
                cache_miss_count += 1;
                executor_subset.extend(partition_subset);
            } else {
                is_partial_hit = true
            }
        }

        Self {
            document: plan.document,
            cache_partitions: plan.cache_partitions,
            cache_keys: fetch_phase.cache_keys,
            cache_entries: fetch_phase.cache_entries,
            executor_subset,
            cache_miss_count,
            is_partial_hit,
            has_nocache_partition,
            request_cache_control: fetch_phase.request_cache_control,
        }
    }

    pub fn query(&self) -> String {
        self.executor_subset
            .as_display(&self.document)
            .include_query_name()
            .to_string()
    }

    pub fn handle_response(self, mut response: QueryResponse, errors: bool) -> (Response, Option<CacheUpdatePhase>) {
        let mut keys_to_write = Vec::with_capacity(self.cache_miss_count);

        // I'd really like to avoid cloning this, but time is not on my side.
        // Going to clone before the updates to make it quicker, may restructure things later
        // to avoid this.
        let update_respones = response.clone();

        let mut response_max_age = MaxAge::default();

        if self.has_nocache_partition {
            // If any portion of our response can't be cached we set the maxAge to none
            response_max_age.set_none();
        }

        for (index, (entry, key)) in self.cache_entries.into_iter().zip(self.cache_keys).enumerate() {
            match entry {
                Entry::Hit(hit, max_age) => {
                    merge_json(&mut response, hit);

                    response_max_age.merge(max_age);
                }
                Entry::Stale(stale) => {
                    // TODO: Also want to issue an update instruction here, but going to do that
                    // in GB-6804
                    merge_json(&mut response, stale.value);

                    // This entry was stale so clear the current maxAge until we have revalidated
                    response_max_age.set_none();
                }
                Entry::Miss if key.is_some() => {
                    response_max_age.merge(Duration::from_secs(self.cache_partitions[index].0.max_age as u64));
                    keys_to_write.push((key.unwrap(), PartitionIndex(index)));
                }
                Entry::Miss => {
                    response_max_age.merge(Duration::from_secs(self.cache_partitions[index].0.max_age as u64));
                }
            }
        }

        let mut update_phase = None;
        if !keys_to_write.is_empty() && !errors && self.request_cache_control.should_write_to_cache {
            // If there are errors we _do not_ want to write to the cache,

            update_phase = Some(CacheUpdatePhase::new(
                self.document,
                self.cache_partitions,
                keys_to_write,
                update_respones,
            ));
        }

        let response = if self.is_partial_hit {
            Response::partial_hit(response, response_max_age)
        } else {
            Response::miss(response, response_max_age)
        };

        (response, update_phase)
    }
}

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
