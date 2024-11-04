#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{Context, Error, ErrorResponse, Headers},
    exports::component::grafbase::gateway_request,
};

struct Component;

impl gateway_request::Guest for Component {
    fn on_gateway_request(_: Context, _: Headers) -> Result<(), ErrorResponse> {
        let error = Error {
            message: String::from("not found"),
            extensions: vec![(String::from("my"), String::from("extension"))],
        };

        Err(ErrorResponse {
            status_code: 403,
            errors: vec![error],
        })
    }
}

bindings::export!(Component with_types_in bindings);
