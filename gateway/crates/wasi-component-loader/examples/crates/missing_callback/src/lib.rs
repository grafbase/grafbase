#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, GatewayRequest, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_subgraph_request(headers: Headers, _: GatewayRequest) -> Result<(), ErrorResponse> {
        headers.set("direct", "call").unwrap();

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
