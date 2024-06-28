#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{Context, ErrorResponse, Headers, SharedContext},
    exports::component::grafbase::{authorization, gateway_request},
};

struct Component;

impl gateway_request::Guest for Component {
    fn on_gateway_request(context: Context, headers: Headers) -> Result<(), ErrorResponse> {
        if let Ok(Some(auth_header)) = headers.get("Authorization") {
            context.set("entitlement", &auth_header);
        }

        Ok(())
    }
}

impl authorization::Guest for Component {
    fn authorized(context: SharedContext, input: Vec<String>) -> Result<Vec<Option<ErrorResponse>>, ErrorResponse> {
        let auth_header = context.get("entitlement");
        let mut result = Vec::with_capacity(input.len());

        for input in input {
            if Some(input) == auth_header {
                result.push(None);
            } else {
                result.push(Some(ErrorResponse {
                    message: String::from("not authorized"),
                    extensions: Vec::new(),
                }))
            }
        }

        Ok(result)
    }
}

bindings::export!(Component with_types_in bindings);
