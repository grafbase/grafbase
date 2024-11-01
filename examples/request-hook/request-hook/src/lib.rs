use bindings::{
    component::grafbase::types::{Context, Error, ErrorResponse, Headers},
    exports::component::grafbase::gateway_request::Guest,
};

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    fn on_gateway_request(_: Context, headers: Headers) -> Result<(), ErrorResponse> {
        match headers.get("x-custom").as_deref() {
            Some("secret") => Ok(()),
            _ => {
                let error = Error {
                    extensions: vec![],
                    message: String::from("access denied"),
                };

                Err(ErrorResponse {
                    status_code: 403,
                    errors: vec![error],
                })
            }
        }
    }
}

bindings::export!(Component with_types_in bindings);
