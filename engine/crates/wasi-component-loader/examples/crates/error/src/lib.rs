#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(_: Headers) -> Result<(), ErrorResponse> {
        Err(ErrorResponse {
            message: String::from("not found"),
            extensions: vec![(String::from("my"), String::from("extension"))],
        })
    }
}

bindings::export!(Component with_types_in bindings);
