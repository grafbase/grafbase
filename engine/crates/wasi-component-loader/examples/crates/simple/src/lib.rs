#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{Context, Error, Headers},
    exports::component::grafbase::gateway_request,
};

struct Component;

impl gateway_request::Guest for Component {
    fn on_gateway_request(context: Context, headers: Headers) -> Result<(), Error> {
        headers.set("direct", "call").unwrap();

        assert_eq!(Ok(Some("call".to_string())), headers.get("direct"));

        if let Ok(var) = std::env::var("GRAFBASE_WASI_TEST") {
            headers.set("fromEnv", &var).unwrap();
        }

        assert_eq!(Some("lol".to_string()), context.get("kekw"));

        context.set("call", "direct");

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
