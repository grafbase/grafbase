use bindings::exports::component::grafbase::{authorization, gateway_request, responses, subgraph_request};

#[allow(warnings)]
mod bindings;

struct Component;

/// Remove this implementation and the gateway-request export from the wit file, if not implementing the hooks.
impl gateway_request::Guest for Component {
    fn on_gateway_request(
        context: gateway_request::Context,
        headers: gateway_request::Headers,
    ) -> Result<(), gateway_request::Error> {
        todo!()
    }
}

/// Remove this implementation and the subgraph-request export from the wit file, if not implementing the hooks.
impl subgraph_request::Guest for Component {
    fn on_subgraph_request(
        context: subgraph_request::SharedContext,
        subgraph_name: String,
        method: String,
        url: String,
        headers: subgraph_request::Headers,
    ) -> Result<(), subgraph_request::Error> {
        todo!()
    }
}

/// Remove this implementation and the authorization export from the wit file, if not implementing the hooks.
impl authorization::Guest for Component {
    fn authorize_edge_pre_execution(
        context: authorization::SharedContext,
        definition: authorization::EdgeDefinition,
        arguments: String,
        metadata: String,
    ) -> Result<(), authorization::Error> {
        todo!()
    }

    fn authorize_node_pre_execution(
        context: authorization::SharedContext,
        definition: authorization::NodeDefinition,
        metadata: String,
    ) -> Result<(), authorization::Error> {
        todo!()
    }

    fn authorize_parent_edge_post_execution(
        context: authorization::SharedContext,
        definition: authorization::EdgeDefinition,
        parents: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), authorization::Error>> {
        todo!()
    }

    fn authorize_edge_node_post_execution(
        context: authorization::SharedContext,
        definition: authorization::EdgeDefinition,
        nodes: Vec<String>,
        metadata: String,
    ) -> Vec<Result<(), authorization::Error>> {
        todo!()
    }

    fn authorize_edge_post_execution(
        context: authorization::SharedContext,
        definition: authorization::EdgeDefinition,
        edges: Vec<(String, Vec<String>)>,
        metadata: String,
    ) -> Vec<Result<(), authorization::Error>> {
        todo!()
    }
}

/// Remove this implementation and the responses export from the wit file, if not implementing the hooks.
impl responses::Guest for Component {
    fn on_subgraph_response(context: responses::SharedContext, request: responses::ExecutedSubgraphRequest) -> Vec<u8> {
        todo!()
    }

    fn on_operation_response(context: responses::SharedContext, request: responses::ExecutedOperation) -> Vec<u8> {
        todo!()
    }

    fn on_http_response(context: responses::SharedContext, request: responses::ExecutedHttpRequest) {
        todo!()
    }
}

bindings::export!(Component with_types_in bindings);
