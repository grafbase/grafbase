#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{ErrorResponse, SharedContext},
    exports::component::grafbase::authorization,
};

struct Component;

impl authorization::Guest for Component {
    fn authorized(_: SharedContext, _: String, _: Vec<String>) -> Result<Vec<Option<ErrorResponse>>, ErrorResponse> {
        Err(ErrorResponse {
            message: String::from("not implemented"),
            extensions: Vec::new(),
        })
    }
}

bindings::export!(Component with_types_in bindings);
