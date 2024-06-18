#[allow(warnings)]
mod bindings;

use bindings::{ErrorResponse, GatewayRequest, Guest, Headers};

struct Component;

impl Guest for Component {
    fn on_gateway_request(headers: Headers, _: GatewayRequest) -> Result<(), ErrorResponse> {
        match std::fs::read_to_string("./contents.txt") {
            Ok(contents) => headers.set("READ_CONTENTS", &contents).unwrap(),
            Err(e) => eprintln!("error reading file contents: {}", e.to_string()),
        }

        if let Err(e) = std::fs::write("./guest_write.txt", "answer") {
            eprintln!("error writing file contents: {}", e.to_string());
        }

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
