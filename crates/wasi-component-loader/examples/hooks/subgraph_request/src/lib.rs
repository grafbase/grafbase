use grafbase_hooks::{Error, Hooks, SharedContext, SubgraphRequest, grafbase_hooks, host_io::http::HttpMethod};

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
        subgraph_request: SubgraphRequest,
    ) -> Result<(), Error> {
        use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

        if context.get("should-fail").is_some() {
            return Err(Error {
                message: "failure".to_string(),
                extensions: Vec::new(),
            });
        }

        let everything = serde_json::to_vec(&serde_json::json!({
            "subgraph_name": subgraph_name,
            "method": subgraph_request.method().as_str(),
            "url": subgraph_request.url(),
            "headers": subgraph_request.headers().entries()
        }))
        .unwrap();

        let encoded = URL_SAFE_NO_PAD.encode(everything);

        subgraph_request.headers().set("everything", &encoded).unwrap();
        let _ = subgraph_request.set_url("https://dark-onion.web");
        subgraph_request.set_method(HttpMethod::Trace);

        Ok(())
    }
}

grafbase_hooks::register_hooks!(Component);
