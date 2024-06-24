#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, Guest, Context, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(context: Context, _: Headers) -> Result<(), ErrorResponse> {
        let address = std::env::var("MOCK_SERVER_ADDRESS").unwrap();
        let response = waki::Client::new().get(&address).send().unwrap().body().unwrap();
        let body = String::from_utf8(response).unwrap();

        context.set("HTTP_RESPONSE", &body);

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
