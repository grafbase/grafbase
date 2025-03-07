use grafbase_hooks::{Context, ErrorResponse, Headers, Hooks, grafbase_hooks, register_hooks};

struct MyHooks;

#[grafbase_hooks]
impl Hooks for MyHooks {
    fn new() -> Self
    where
        Self: Sized,
    {
        MyHooks
    }

    fn on_gateway_request(&mut self, context: Context, url: String, headers: Headers) -> Result<(), ErrorResponse> {
        headers.set("direct", "call").unwrap();

        assert_eq!(Some("call".to_string()), headers.get("direct"));

        if let Ok(var) = std::env::var("GRAFBASE_WASI_TEST") {
            headers.set("fromEnv", &var).unwrap();
        }

        assert_eq!(Some("lol".to_string()), context.get("kekw"));

        context.set("call", "direct");
        context.set("url", &url);

        Ok(())
    }
}

register_hooks!(MyHooks);
