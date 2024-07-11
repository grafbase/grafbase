use std::{collections::BTreeSet, time::Duration};

use graph_entities::{QueryResponse, QueryResponseNode};
use query_path::QueryPathSegment;
use runtime::cache::Entry;

use crate::{
    output::{self, handle_initial_response, Object, ObjectShape, OutputShapes, OutputStore, Value},
    planning::defers::DeferId,
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
    fully_cached_defers: Vec<DeferId>,
    output: Option<OutputStore>,
}

impl StreamingExecutionPhase {
    pub(super) fn new(execution_phase: ExecutionPhase) -> StreamingExecutionPhase {
        let shapes = OutputShapes::new(&execution_phase.plan, execution_phase.type_relationships.as_ref());

        let fully_cached_defers = execution_phase
            .plan
            .defers()
            .filter(|defer| !execution_phase.executor_subset.contains_selection(defer.spread_id()))
            .map(|defer| defer.id)
            .collect();

        StreamingExecutionPhase {
            execution_phase,
            shapes,
            keys_to_write: vec![],
            seen_errors: false,
            fully_cached_defers,
            output: None,
        }
    }

    pub fn query(&self) -> String {
        self.execution_phase.query()
    }

    pub fn record_initial_response(&mut self, response: QueryResponse, errors: bool) -> QueryResponse {
        self.seen_errors = errors;

        let root_shape = self.shapes.root();

        let (mut store, mut active_defers) = handle_initial_response(
            response,
            &self.shapes,
            root_shape,
            self.execution_phase.type_relationships.as_ref(),
        );
        active_defers.extend(self.fully_cached_defers.iter().copied());

        let mut response_max_age = MaxAge::default();

        if self.execution_phase.has_nocache_partition() {
            // If any portion of our response can't be cached we set the maxAge to none
            response_max_age.set_none();
        }

        let cache_entries = self.execution_phase.cache_entries.iter_mut();
        let cache_keys = std::mem::take(&mut self.execution_phase.cache_keys);

        let type_relationships = self.execution_phase.type_relationships.as_ref();

        for (index, (entry, key)) in cache_entries.zip(cache_keys).enumerate() {
            match entry {
                Entry::Hit(hit, max_age) => {
                    store.merge_cache_entry(hit, &self.shapes, &active_defers, type_relationships);

                    response_max_age.merge(*max_age);
                }
                Entry::Stale(stale) => {
                    // TODO: Also want to issue an update instruction here, but going to do that
                    // in GB-6804
                    store.merge_cache_entry(&mut stale.value, &self.shapes, &active_defers, type_relationships);

                    // This entry was stale so clear the current maxAge until we have revalidated
                    response_max_age.set_none();
                }
                Entry::Miss if key.is_some() => {
                    response_max_age.merge(Duration::from_secs(
                        self.execution_phase.plan.cache_partitions[index].0.max_age as u64,
                    ));
                    self.keys_to_write.push((key.unwrap(), PartitionIndex(index)));
                }
                Entry::Miss => {
                    response_max_age.merge(Duration::from_secs(
                        self.execution_phase.plan.cache_partitions[index].0.max_age as u64,
                    ));
                }
            }
        }

        let return_value = store
            .reader(&self.shapes)
            .map(|object| object.into_query_response(false))
            .unwrap_or_default();

        self.output = Some(store);

        return_value
    }

    pub fn record_incremental_response(
        &mut self,
        label: Option<&str>,
        path: &[&QueryPathSegment],
        data: QueryResponse,
        errors: bool,
    ) -> QueryResponse {
        if self.output.is_none() {
            todo!("GB-6966");
        }
        let Some(destination_object) = self.object_at_path(path) else {
            todo!("GB-6966");
        };
        let Some(defer) = self.lookup_defer(label, destination_object, &data) else {
            todo!("GB-6966");
        };
        let destination_object_id = destination_object.id;
        let output = self.output.as_mut().unwrap();

        if errors {
            self.seen_errors = true;
        }

        let active_nested_defers = output.merge_incremental_payload(
            destination_object_id,
            data,
            &self.shapes,
            self.execution_phase.type_relationships.as_ref(),
        );

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

            let type_relationships = self.execution_phase.type_relationships.as_ref();

            for mut value in cache_values {
                output.merge_specific_defer_from_cache_entry(
                    &mut value,
                    &self.shapes,
                    defer,
                    &active_nested_defers,
                    type_relationships,
                );
            }
        }

        output
            .read_object(&self.shapes, destination_object_id)
            .into_query_response(false)
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
                        self.execution_phase.plan.document,
                        self.execution_phase.plan.cache_partitions,
                        self.keys_to_write,
                        root.into_query_response(true),
                    ));
                }
            }
        }

        update_phase
    }

    fn lookup_defer(
        &self,
        label: Option<&str>,
        destination_object: Object<'_>,
        data: &QueryResponse,
    ) -> Option<DeferId> {
        if let Some(label) = label {
            return Some(
                self.execution_phase
                    .plan
                    .defers()
                    .find(|defer| defer.label() == Some(label))?
                    .id,
            );
        }
        let mut possible_defers = self.shapes.defers_for_object(destination_object.shape().id);
        if possible_defers.len() <= 1 {
            // The easy case
            return possible_defers.next();
        }

        // If there's no label and multiple possible defers in this object we'll have to examine
        // the output to figure out what defer this is.  Urgh.
        let possible_defers = possible_defers.collect::<BTreeSet<_>>();
        let mut field_stack = vec![(data.root?, ObjectShape::Concrete(destination_object.shape()))];

        while let Some((id, object_shape)) = field_stack.pop() {
            match data.get_node(id)? {
                QueryResponseNode::Container(container) => {
                    let concrete_shape = match object_shape {
                        ObjectShape::Concrete(shape) => shape,
                        ObjectShape::Polymorphic(shape) => {
                            let Some(typename) =
                                container.child("__typename").and_then(|id| data.get_node(id)?.as_str())
                            else {
                                todo!("GB-6966")
                            };
                            shape
                                .concrete_shape_for_typename(typename, self.execution_phase.type_relationships.as_ref())
                        }
                    };

                    for (name, src_id) in container.iter() {
                        let Some(field_shape) = concrete_shape.field(name.as_str()) else {
                            continue;
                        };
                        if let Some(id) = field_shape.defer_id() {
                            if possible_defers.contains(&id) {
                                return Some(id);
                            }
                        }
                        if let Some(subselection_shape) = field_shape.subselection_shape() {
                            field_stack.push((*src_id, subselection_shape))
                        }
                    }
                }
                QueryResponseNode::List(list) => {
                    field_stack.extend(list.iter().map(|id| (id, object_shape)));
                }
                QueryResponseNode::Primitive(_) => {}
            }
        }

        // Ok, so we can have defers with no unique fields where users do
        // something weird like nest defers immediately inside other defers.
        //
        // It's quite difficult to determine the defer in those cases.
        //
        // But given that the alternative is to error out, lets just guess
        // that it's the first one we encountered.
        possible_defers.first().copied()
    }

    fn object_at_path<'b>(&'b self, path: &[&QueryPathSegment]) -> Option<Object<'b>> {
        let mut value = Value::Object(self.output.as_ref()?.reader(&self.shapes)?);

        for segment in path {
            match (segment, value) {
                (_, output::Value::Null) => return None,
                (QueryPathSegment::Index(index), output::Value::List(list)) => value = list.get_index(*index)?,
                (QueryPathSegment::Field(field_name), output::Value::Object(object)) => {
                    value = object.field(field_name.as_str())?;
                }
                _ => return None,
            }
        }

        match value {
            Value::Object(object) => Some(object),
            _ => None,
        }
    }
}
