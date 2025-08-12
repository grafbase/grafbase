use crate::InstanceState;

pub use super::grafbase::sdk::event_types::*;

impl Host for InstanceState {}

pub(crate) fn convert_event(state: &mut InstanceState, event: event_queue::Event) -> wasmtime::Result<Event> {
    let event = match event {
        event_queue::Event::Operation(op) => Event::Operation(op.into()),
        event_queue::Event::Subgraph(event) => convert_subgraph_event(state, event)?,
        event_queue::Event::Http(http) => Event::Http(http.into()),
        event_queue::Event::Extension(ext) => Event::Extension(ext.into()),
    };

    Ok(event)
}

fn convert_subgraph_event(
    state: &mut InstanceState,
    subgraph: event_queue::ExecutedSubgraphRequest,
) -> wasmtime::Result<Event> {
    let mut executions = Vec::new();
    for execution in subgraph.executions {
        let execution = match execution {
            event_queue::RequestExecution::InternalServerError => SubgraphRequestExecutionKind::InternalServerError,
            event_queue::RequestExecution::RequestError => SubgraphRequestExecutionKind::RequestError,
            event_queue::RequestExecution::RateLimited => SubgraphRequestExecutionKind::RateLimited,
            event_queue::RequestExecution::Response(resp) => {
                let response_headers = Headers::from(resp.headers);
                let response_headers = state.resources.push(response_headers)?;

                SubgraphRequestExecutionKind::Response(SubgraphResponse {
                    connection_time_ns: resp.connection_time.as_nanos() as u64,
                    response_time_ns: resp.response_time.as_nanos() as u64,
                    status_code: resp.status.as_u16(),
                    response_headers,
                })
            }
        };

        executions.push(execution);
    }
    let event = ExecutedSubgraphRequest {
        subgraph_name: subgraph.subgraph_name,
        method: subgraph.method.into(),
        url: subgraph.url,
        executions,
        cache_status: subgraph.cache_status.into(),
        total_duration_ns: subgraph.total_duration.as_nanos() as u64,
        has_errors: subgraph.has_errors,
    };
    Ok(Event::Subgraph(event))
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
            operation_type: value.operation_type.into(),
            complexity: value.complexity,
            has_deprecated_fields: value.has_deprecated_fields,
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

impl From<event_queue::OperationType> for OperationType {
    fn from(value: event_queue::OperationType) -> Self {
        match value {
            event_queue::OperationType::Query => OperationType::Query,
            event_queue::OperationType::Mutation => OperationType::Mutation,
            event_queue::OperationType::Subscription => OperationType::Subscription,
        }
    }
}
