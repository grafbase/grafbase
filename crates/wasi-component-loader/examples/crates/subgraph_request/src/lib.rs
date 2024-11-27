use grafbase_hooks::{grafbase_hooks, Error, Headers, Hooks, SharedContext};

struct Component;

#[grafbase_hooks]
impl Hooks for Component {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn on_subgraph_request(
        &mut self,
        context: SharedContext,
        subgraph_name: String,
        method: String,
        url: String,
        headers: Headers,
    ) -> Result<(), Error> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        if context.get("should-fail").is_some() {
            return Err(Error {
                message: "failure".to_string(),
                extensions: Vec::new(),
            });
        }

        let everything = serde_json::to_vec(&serde_json::json!({
            "subgraph_name": subgraph_name,
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

grafbase_hooks::register_hooks!(Component);
