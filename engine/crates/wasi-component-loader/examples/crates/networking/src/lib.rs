#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(headers: Headers) -> Result<(), ErrorResponse> {
        let address = std::env::var("MOCK_SERVER_ADDRESS").unwrap();
        let response = waki::Client::new().get(&address).send().unwrap().body().unwrap();
        let body = String::from_utf8(response).unwrap();

        headers.set("HTTP_RESPONSE", &body).unwrap();

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
