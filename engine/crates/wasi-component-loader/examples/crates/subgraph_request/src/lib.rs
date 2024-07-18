#[allow(warnings)]
mod bindings;

use bindings::{component::grafbase::types::Error, exports::component::grafbase::subgraph_request};

struct Component;

impl subgraph_request::Guest for Component {
    fn on_subgraph_request(
        context: subgraph_request::SharedContext,
        method: String,
        url: String,
        headers: subgraph_request::Headers,
    ) -> Result<(), subgraph_request::Error> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        if context.get("should-fail").is_some() {
            return Err(Error {
                message: "failure".to_string(),
                extensions: Vec::new(),
            });
        }

        let everything = serde_json::to_vec(&serde_json::json!({
            "method": method,
            "url": url,
            "headers": headers.entries()
        }))
        .unwrap();

        let encoded = URL_SAFE_NO_PAD.encode(everything);

        headers.set("everything", &encoded).unwrap();

        Ok(())
    }
}

bindings::export!(Component with_types_in bindings);
