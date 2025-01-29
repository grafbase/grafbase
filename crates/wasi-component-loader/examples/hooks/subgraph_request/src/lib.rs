use grafbase_hooks::{grafbase_hooks, Error, Headers, Hooks, SharedContext, SubgraphRequest};

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
        headers: Headers,
        subgraph_request: SubgraphRequest,
    ) -> Result<(), Error> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        if context.get("should-fail").is_some() {
            return Err(Error {
                message: "failure".to_string(),
                extensions: Vec::new(),
            });
        }

        let everything = serde_json::to_vec(&serde_json::json!({
            "subgraph_name": subgraph_request.subgraph_name(),
            "method": subgraph_request.method().as_str(),
            "url": subgraph_request.url(),
            "headers": headers.entries()
        }))
        .unwrap();

        let encoded = URL_SAFE_NO_PAD.encode(everything);

        headers.set("everything", &encoded).unwrap();

        Ok(())
    }
}

grafbase_hooks::register_hooks!(Component);
