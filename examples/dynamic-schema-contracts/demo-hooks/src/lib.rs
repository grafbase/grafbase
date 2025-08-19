use grafbase_sdk::{
    host_io::http::Method,
    types::{
        AuthorizedOperationContext, Configuration, Error, ErrorResponse, Headers, HttpRequestParts, OnRequestOutput,
    },
    HooksExtension,
};
use serde_json::json;

#[derive(HooksExtension)]
struct DemoHooks;

impl HooksExtension for DemoHooks {
    fn new(_config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    #[allow(refining_impl_trait)]
    fn on_request(
        &mut self,
        _url: &str,
        _method: Method,
        headers: &mut Headers,
    ) -> Result<OnRequestOutput, ErrorResponse> {
        // Check API key header to determine role and clinic access
        let contract_key = if let Some(api_key) = headers.get("x-api-key") {
            let key_str = api_key.to_str().unwrap_or("");

            match key_str {
                k if k.starts_with("patient-") => {
                    json!({
                        "includedTags": ["public", "patient-facing", "clinic-a"]
                    })
                }
                k if k.starts_with("doctor-") => {
                    json!({
                        "includedTags": ["public", "patient-facing", "doctor-access", "clinic-a"]
                    })
                }
                // Billing staff - no clinic access, only insurance data
                k if k.starts_with("billing-") => {
                    json!({
                        "includedTags": ["public", "billing"]
                    })
                }
                // Admin - full access to everything
                k if k.starts_with("admin-") => {
                    json!({
                        "includedTags": ["public", "patient-facing", "doctor-access", "billing", "admin-only", "clinic-a", "clinic-b"]
                    })
                }
                // Unknown API key - public only
                _ => {
                    json!({
                        "includedTags": ["public"]
                    })
                }
            }
        } else {
            // No API key - public access only
            json!({
                "includedTags": ["public"]
            })
        };

        // Get clinic from x-clinic header, defaulting to "a"
        let clinic = if let Some(header_value) = headers.get("x-clinic") {
            header_value.to_str().unwrap_or("a").to_string()
        } else {
            "a".to_string()
        };

        // Set the clinic in the hooks context
        let context = json!({ "clinic": clinic });
        let context_bytes = serde_json::to_vec(&context).unwrap_or_default();

        Ok(OnRequestOutput::new()
            .contract_key(contract_key.to_string())
            .context(context_bytes))
    }

    fn on_graphql_subgraph_request(
        &mut self,
        ctx: &AuthorizedOperationContext,
        subgraph_name: &str,
        parts: &mut HttpRequestParts,
    ) -> Result<(), Error> {
        // Only route clinic-a and clinic-b subgraphs
        if subgraph_name == "clinic-a" || subgraph_name == "clinic-b" {
            // Get the hooks context that was set in on_request
            let context_bytes = ctx.hooks_context();

            // Parse the context to get the clinic selection
            if let Ok(context_str) = std::str::from_utf8(&context_bytes) {
                if let Ok(context_json) = serde_json::from_str::<serde_json::Value>(context_str) {
                    if let Some(clinic) = context_json.get("clinic").and_then(|v| v.as_str()) {
                        // Route to clinic-b if specified, otherwise keep default routing
                        if clinic == "b" && subgraph_name == "clinic-a" {
                            // Override the URL to route to clinic-b instead
                            parts.url = "http://clinic-b:4003/graphql".to_string();
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
