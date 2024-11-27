use grafbase_hooks::{grafbase_hooks, Context, Error, ErrorResponse, Headers, Hooks};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, _: Context, headers: Headers) -> Result<(), ErrorResponse> {
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

grafbase_hooks::register_hooks!(Component);
