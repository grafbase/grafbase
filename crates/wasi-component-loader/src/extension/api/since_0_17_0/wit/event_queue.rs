use wasmtime::component::Resource;

use crate::{InstanceState, resources::EventQueueProxy};

pub use super::grafbase::sdk::event_queue::*;

impl Host for InstanceState {}

impl HostEventQueue for InstanceState {
    async fn pop(&mut self, self_: Resource<EventQueueProxy>) -> wasmtime::Result<Option<Event>> {
        let this = self.resources.get(&self_)?;
        Ok(this.0.pop().map(Into::into))
    }

    async fn drop(&mut self, res: Resource<EventQueueProxy>) -> wasmtime::Result<()> {
        self.resources.delete(res)?;

        Ok(())
    }
}

impl From<event_queue::Event> for Event {
    fn from(value: event_queue::Event) -> Self {
        match value {
            event_queue::Event::Operation(op) => Event::Operation(op.into()),
            event_queue::Event::Subgraph(subgraph) => Event::Subgraph(subgraph.into()),
            event_queue::Event::Http(http) => Event::Http(http.into()),
            event_queue::Event::Extension(ext) => Event::Extension(ext.into()),
        }
    }
}

impl From<event_queue::ExecutedOperation> for ExecutedOperation {
    fn from(value: event_queue::ExecutedOperation) -> Self {
        ExecutedOperation {
            name: value.name,
            document: value.document.to_string(),
            prepare_duration_ns: value.prepare_duration.as_nanos() as u64,
            cached_plan: value.cached_plan,
            duration_ns: value.duration.as_nanos() as u64,
            status: value.status.into(),
        }
    }
}

impl From<grafbase_telemetry::graphql::GraphqlResponseStatus> for GraphqlResponseStatus {
    fn from(value: grafbase_telemetry::graphql::GraphqlResponseStatus) -> Self {
        match value {
            grafbase_telemetry::graphql::GraphqlResponseStatus::Success => GraphqlResponseStatus::Success,
            grafbase_telemetry::graphql::GraphqlResponseStatus::FieldError { count, data_is_null } => {
                GraphqlResponseStatus::FieldError(FieldError { count, data_is_null })
            }
            grafbase_telemetry::graphql::GraphqlResponseStatus::RequestError { count } => {
                GraphqlResponseStatus::RequestError(RequestError { count })
            }
            grafbase_telemetry::graphql::GraphqlResponseStatus::RefusedRequest => GraphqlResponseStatus::RefusedRequest,
        }
    }
}

impl From<event_queue::ExecutedSubgraphRequest> for ExecutedSubgraphRequest {
    fn from(value: event_queue::ExecutedSubgraphRequest) -> Self {
        // We need to handle response headers - for now we'll create an empty Headers resource
        // This would need proper implementation based on your Headers resource type
        let headers_resource = wasmtime::component::Resource::new_own(0);

        ExecutedSubgraphRequest {
            subgraph_name: value.subgraph_name,
            method: value.method.into(),
            url: value.url,
            executions: value.executions.into_iter().map(Into::into).collect(),
            cache_status: value.cache_status.into(),
            total_duration_ns: value.total_duration.as_nanos() as u64,
            has_errors: value.has_errors,
            response_headers: headers_resource,
        }
    }
}

impl From<event_queue::RequestExecution> for SubgraphRequestExecutionKind {
    fn from(value: event_queue::RequestExecution) -> Self {
        match value {
            event_queue::RequestExecution::InternalServerError => SubgraphRequestExecutionKind::InternalServerError,
            event_queue::RequestExecution::RequestError => SubgraphRequestExecutionKind::RequestError,
            event_queue::RequestExecution::RateLimited => SubgraphRequestExecutionKind::RateLimited,
            event_queue::RequestExecution::Response(response) => {
                SubgraphRequestExecutionKind::Response(SubgraphResponse {
                    connection_time_ns: response.connection_time.as_nanos() as u64,
                    response_time_ns: response.response_time.as_nanos() as u64,
                    status_code: response.status.as_u16(),
                })
            }
        }
    }
}

impl From<event_queue::CacheStatus> for CacheStatus {
    fn from(value: event_queue::CacheStatus) -> Self {
        match value {
            event_queue::CacheStatus::Hit => CacheStatus::Hit,
            event_queue::CacheStatus::PartialHit => CacheStatus::PartialHit,
            event_queue::CacheStatus::Miss => CacheStatus::Miss,
        }
    }
}

impl From<event_queue::ExecutedHttpRequest> for ExecutedHttpRequest {
    fn from(value: event_queue::ExecutedHttpRequest) -> Self {
        ExecutedHttpRequest {
            method: value.method.into(),
            url: value.url,
            status_code: value.response_status.as_u16(),
        }
    }
}

impl From<event_queue::ExtensionEvent> for ExtensionEvent {
    fn from(value: event_queue::ExtensionEvent) -> Self {
        ExtensionEvent {
            extension_name: value.extension_name,
            event_name: value.event_name,
            data: value.data,
        }
    }
}
