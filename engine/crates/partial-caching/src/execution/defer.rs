use std::time::Duration;

use graph_entities::QueryResponse;
use query_path::QueryPathSegment;
use runtime::cache::Entry;

use crate::{
    output::{self, InitialOutput, OutputShapes, OutputStore},
    response::MaxAge,
    updating::PartitionIndex,
    CacheUpdatePhase,
};

use super::ExecutionPhase;

pub struct StreamingExecutionPhase {
    execution_phase: ExecutionPhase,
    shapes: OutputShapes,
    keys_to_write: Vec<(String, PartitionIndex)>,
    seen_errors: bool,
    output: Option<OutputStore>,
}

impl StreamingExecutionPhase {
    pub(super) fn new(execution_phase: ExecutionPhase) -> StreamingExecutionPhase {
        let shapes = OutputShapes::new(execution_phase.operation());

        StreamingExecutionPhase {
            execution_phase,
            shapes,
            keys_to_write: vec![],
            seen_errors: false,
            output: None,
        }
    }

    pub fn query(&self) -> String {
        self.execution_phase.query()
    }

    pub fn record_initial_response(&mut self, response: QueryResponse, errors: bool) -> QueryResponse {
        self.seen_errors = errors;

        let root_shape = self.shapes.root();

        let InitialOutput {
            mut store,
            active_defers,
        } = InitialOutput::new(response, root_shape);

        let mut response_max_age = MaxAge::default();

        if self.execution_phase.has_nocache_partition {
            // If any portion of our response can't be cached we set the maxAge to none
            response_max_age.set_none();
        }

        let cache_entries = self.execution_phase.cache_entries.iter_mut();
        let cache_keys = std::mem::take(&mut self.execution_phase.cache_keys);

        for (index, (entry, key)) in cache_entries.zip(cache_keys).enumerate() {
            match entry {
                Entry::Hit(hit, max_age) => {
                    store.merge_cache_entry(hit, &self.shapes, &active_defers);

                    response_max_age.merge(*max_age);
                }
                Entry::Stale(stale) => {
                    // TODO: Also want to issue an update instruction here, but going to do that
                    // in GB-6804
                    store.merge_cache_entry(&mut stale.value, &self.shapes, &active_defers);

                    // This entry was stale so clear the current maxAge until we have revalidated
                    response_max_age.set_none();
                }
                Entry::Miss if key.is_some() => {
                    response_max_age.merge(Duration::from_secs(
                        self.execution_phase.cache_partitions[index].0.max_age as u64,
                    ));
                    self.keys_to_write.push((key.unwrap(), PartitionIndex(index)));
                }
                Entry::Miss => {
                    response_max_age.merge(Duration::from_secs(
                        self.execution_phase.cache_partitions[index].0.max_age as u64,
                    ));
                }
            }
        }

        let return_value = store
            .reader(&self.shapes)
            .map(|object| object.into_query_response())
            .unwrap_or_default();

        self.output = Some(store);

        return_value
    }

    pub fn record_incremental_response(
        &mut self,
        defer_label: &str,
        path: &[&QueryPathSegment],
        data: QueryResponse,
        errors: bool,
    ) -> QueryResponse {
        let Some(output) = &mut self.output else {
            todo!("GB-6966");
        };

        if !self.execution_phase.cache_entries.is_empty() {
            // If we still have cache entries, we should merge the rest of them into
            // the store before handling this incremental response
            let cache_values = std::mem::take(&mut self.execution_phase.cache_entries)
                .into_iter()
                .filter_map(|entry| match entry {
                    Entry::Hit(value, _) => Some(value),
                    Entry::Stale(stale) => Some(stale.value),
                    _ => None,
                });

            for mut value in cache_values {
                output.merge_specific_defer_from_cache_entry(&mut value, &self.shapes, defer_label);
            }
        }

        if errors {
            self.seen_errors = true;
        }

        output.merge_incremental_payload(path, data, &self.shapes);

        let Some(object) = output.reader(&self.shapes) else {
            todo!("GB-6966");
        };

        let mut value = crate::output::Value::Object(object);

        for segment in path {
            match (segment, value) {
                (_, output::Value::Null) => return QueryResponse::default(),
                (QueryPathSegment::Index(index), output::Value::List(list)) => {
                    let Some(item) = list.get_index(*index) else {
                        todo!("GB-6966")
                    };
                    value = item;
                }
                (QueryPathSegment::Field(field_name), output::Value::Object(object)) => {
                    let Some(field) = object.field(field_name.as_str()) else {
                        todo!("GB-6966")
                    };
                    value = field;
                }
                _ => todo!("GB-6966"),
            }
        }

        let output::Value::Object(object) = value else {
            todo!("GB-6966")
        };

        object.into_query_response()
    }

    pub fn finish(mut self) -> Option<CacheUpdatePhase> {
        let mut update_phase = None;

        // If there are errors we _do not_ want to write to the cache,
        if !self.keys_to_write.is_empty()
            && !self.seen_errors
            && self.execution_phase.request_cache_control.should_write_to_cache
        {
            if let Some(output) = self.output.take() {
                if let Some(root) = output.reader(&self.shapes) {
                    update_phase = Some(CacheUpdatePhase::new(
                        self.execution_phase.document,
                        self.execution_phase.cache_partitions,
                        self.keys_to_write,
                        root.into_query_response(),
                    ));
                }
            }
        }

        update_phase
    }
}
