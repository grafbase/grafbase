#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, GatewayRequest, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(_: Headers, _: GatewayRequest) -> Result<(), ErrorResponse> {
        Err(ErrorResponse {
            status: Some(404),
            message: String::from("not found"),
        })
    }
}

bindings::export!(Component with_types_in bindings);
