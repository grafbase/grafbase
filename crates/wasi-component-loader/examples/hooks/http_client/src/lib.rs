use grafbase_hooks::{
    Context, ErrorResponse, Headers, Hooks, grafbase_hooks,
    host_io::http::{self, HttpMethod, HttpRequest},
};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, context: Context, _: String, _: Headers) -> Result<(), ErrorResponse> {
        let address = std::env::var("MOCK_SERVER_ADDRESS").unwrap();

        let request = HttpRequest {
            method: HttpMethod::Get,
            url: address,
            headers: Vec::new(),
            body: Vec::new(),
            timeout_ms: None,
        };

        let response = http::execute(&request).unwrap();
        let body = String::from_utf8(response.body).unwrap();

        context.set("HTTP_RESPONSE", &body);

        Ok(())
    }
}

grafbase_hooks::register_hooks!(Component);
