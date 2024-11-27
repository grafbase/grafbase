use grafbase_hooks::{grafbase_hooks, Context, ErrorResponse, Headers, Hooks};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_gateway_request(&mut self, _: Context, _: Headers) -> Result<(), ErrorResponse> {
        Ok(())
    }
}

grafbase_hooks::register_hooks!(Component);
