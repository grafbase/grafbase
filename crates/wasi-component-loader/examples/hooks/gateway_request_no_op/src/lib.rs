use grafbase_hooks::{Context, ErrorResponse, Headers, Hooks, grafbase_hooks};

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
        Ok(())
    }
}

grafbase_hooks::register_hooks!(Component);
