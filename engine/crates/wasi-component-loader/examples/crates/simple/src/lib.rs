#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(headers: Headers) -> Result<(), ErrorResponse> {
        headers.set("direct", "call").unwrap();

        assert_eq!(Ok(Some("call".to_string())), headers.get("direct"));

        if let Ok(var) = std::env::var("GRAFBASE_WASI_TEST") {
            headers.set("fromEnv", &var).unwrap();
        }

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
