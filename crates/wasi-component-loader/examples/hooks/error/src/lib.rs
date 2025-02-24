use grafbase_hooks::{Context, Error, ErrorResponse, Headers, Hooks, grafbase_hooks};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, _: Context, _: String, _: Headers) -> Result<(), ErrorResponse> {
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

grafbase_hooks::register_hooks!(Component);
