#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{Context, ErrorResponse, Headers},
    exports::component::grafbase::gateway_request,
};

struct Component;

impl gateway_request::Guest for Component {
    fn on_gateway_request(_: Context, _: Headers) -> Result<(), ErrorResponse> {
        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
