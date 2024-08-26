use bindings::{
    component::grafbase::types::SubgraphResponse,
    exports::component::grafbase::responses::{
        ExecutedGatewayRequest, ExecutedHttpRequest, ExecutedSubgraphRequest, Guest, Operation, SharedContext,
    },
};

#[allow(warnings)]
mod bindings;

struct Component;

#[derive(serde::Serialize, serde::Deserialize)]
struct SubgraphInfo {
    subgraph_name: String,
    method: String,
    url: String,
    connection_times: Vec<u64>,
    response_times: Vec<u64>,
    status_codes: Vec<u16>,
    total_duration: u64,
    has_errors: bool,
    cached: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FieldError {
    count: u64,
    data_is_null: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RequestError {
    count: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum GraphqlResponseStatus {
    Success,
    FieldError(FieldError),
    RequestError(RequestError),
    RefusedRequest,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct OperationInfo {
    name: Option<String>,
    document: String,
    prepare_duration: u64,
    cached: bool,
    duration: u64,
    status: GraphqlResponseStatus,
    subgraphs: Vec<SubgraphInfo>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AuditInfo {
    method: String,
    url: String,
    status_code: u16,
    operations: Vec<OperationInfo>,
}

impl Guest for Component {
    fn on_subgraph_response(_: SharedContext, request: ExecutedSubgraphRequest) -> Vec<u8> {
        let ExecutedSubgraphRequest {
            subgraph_name,
            method,
            url,
            response,
            total_duration,
            has_errors,
        } = request;

        let connection_times = match response {
            SubgraphResponse::Responses(ref responses) => responses.iter().map(|r| r.connection_time).collect(),
            SubgraphResponse::Cached => Vec::new(),
        };

        let response_times = match response {
            SubgraphResponse::Responses(ref responses) => responses.iter().map(|r| r.response_time).collect(),
            SubgraphResponse::Cached => Vec::new(),
        };

        let status_codes = match response {
            SubgraphResponse::Responses(ref responses) => responses.iter().map(|r| r.status_code).collect(),
            SubgraphResponse::Cached => Vec::new(),
        };

        let info = SubgraphInfo {
            subgraph_name,
            method,
            url,
            connection_times,
            response_times,
            status_codes,
            total_duration,
            has_errors,
            cached: false,
        };

        serde_json::to_vec(&info).unwrap()
    }

    fn on_gateway_response(_: SharedContext, operation: Operation, request: ExecutedGatewayRequest) -> Vec<u8> {
        let info = OperationInfo {
            name: operation.name,
            document: operation.document,
            prepare_duration: operation.prepare_duration,
            cached: operation.cached,
            duration: request.duration,
            status: match request.status {
                bindings::component::grafbase::types::GraphqlResponseStatus::Success => GraphqlResponseStatus::Success,
                bindings::component::grafbase::types::GraphqlResponseStatus::FieldError(e) => {
                    GraphqlResponseStatus::FieldError(FieldError {
                        count: e.count,
                        data_is_null: e.data_is_null,
                    })
                }
                bindings::component::grafbase::types::GraphqlResponseStatus::RequestError(e) => {
                    GraphqlResponseStatus::RequestError(RequestError { count: e.count })
                }
                bindings::component::grafbase::types::GraphqlResponseStatus::RefusedRequest => {
                    GraphqlResponseStatus::RefusedRequest
                }
            },
            subgraphs: request
                .on_subgraph_request_outputs
                .iter()
                .filter_map(|bytes| serde_json::from_slice(bytes).ok())
                .collect(),
        };

        serde_json::to_vec(&info).unwrap()
    }

    fn on_http_response(context: SharedContext, request: ExecutedHttpRequest) {
        let info = AuditInfo {
            method: request.method,
            url: request.url,
            status_code: request.status_code,
            operations: request
                .on_gateway_response_outputs
                .iter()
                .filter_map(|bytes| serde_json::from_slice(bytes).ok())
                .collect(),
        };

        context.log_access(&serde_json::to_vec(&info).unwrap()).unwrap();
    }
}

bindings::export!(Component with_types_in bindings);
