#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, GatewayRequest, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(headers: Headers, request: GatewayRequest) -> Result<(), ErrorResponse> {
        headers.set("direct", "call").unwrap();
        request.set_operation_name(Some("test"));
        request.set_document_id(Some("jest"));

        assert_eq!(Ok(Some("call".to_string())), headers.get("direct"));
        assert_eq!(Some("test".to_string()), request.get_operation_name());
        assert_eq!(Some("jest".to_string()), request.get_document_id());

        if let Ok(var) = std::env::var("GRAFBASE_WASI_TEST") {
            headers.set("fromEnv", &var).unwrap();
        }

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
