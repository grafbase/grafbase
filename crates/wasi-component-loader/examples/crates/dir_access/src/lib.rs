#[allow(warnings)]
mod bindings;

use bindings::{
    component::grafbase::types::{Context, ErrorResponse, Headers},
    exports::component::grafbase::gateway_request,
};

struct Component;

impl gateway_request::Guest for Component {
    fn on_gateway_request(_: Context, headers: Headers) -> Result<(), ErrorResponse> {
        match std::fs::read_to_string("./contents.txt") {
            Ok(contents) => headers.set("READ_CONTENTS", &contents).unwrap(),
            Err(e) => eprintln!("error reading file contents: {e}"),
        }

        if let Err(e) = std::fs::write("./guest_write.txt", "answer") {
            eprintln!("error writing file contents: {e}");
        }

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
