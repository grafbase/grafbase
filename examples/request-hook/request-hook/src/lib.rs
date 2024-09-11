use bindings::exports::component::grafbase::gateway_request::{Context, Error, Guest, Headers};

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    fn on_gateway_request(_: Context, headers: Headers) -> Result<(), Error> {
        match headers.get("x-custom").as_deref() {
            Some("secret") => Ok(()),
            _ => Err(Error {
                extensions: vec![],
                message: String::from("access denied"),
            }),
        }
    }
}

bindings::export!(Component with_types_in bindings);
