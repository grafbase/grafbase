#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, Guest, Context, Headers};

struct Component;

impl Guest for Component {
    fn on_subgraph_request(_: Context, headers: Headers) -> Result<(), ErrorResponse> {
        headers.set("direct", "call").unwrap();

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
