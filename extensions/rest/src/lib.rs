mod selection_filter;
mod types;

use grafbase_sdk::{
    Error, Extension, Resolver, ResolverExtension, SharedContext, Subscription,
    host_io::http::{self, HttpRequest, Url},
    jq_selection::JqSelection,
    types::{Configuration, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};
use types::{Rest, RestEndpoint};

#[derive(ResolverExtension)]
struct RestExtension {
    endpoints: Vec<RestEndpoint>,
    jq_selection: JqSelection,
}

impl Extension for RestExtension {
    fn new(schema_directives: Vec<SchemaDirective>, _: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::<RestEndpoint>::new();

        for directive in schema_directives {
            let endpoint = RestEndpoint {
                subgraph_name: directive.subgraph_name().to_string(),
                args: directive.arguments()?,
            };

            endpoints.push(endpoint);
        }

        endpoints.sort_by(|a, b| {
            let by_name = a.args.name.cmp(&b.args.name);
            let by_subgraph = a.subgraph_name.cmp(&b.subgraph_name);
            by_name.then(by_subgraph)
        });

        Ok(Self {
            endpoints,
            jq_selection: JqSelection::default(),
        })
    }
}

impl RestExtension {
    pub fn get_endpoint(&self, name: &str, subgraph_name: &str) -> Option<&RestEndpoint> {
        self.endpoints
            .binary_search_by(|e| {
                let by_name = e.args.name.as_str().cmp(name);
                let by_subgraph = e.subgraph_name.as_str().cmp(subgraph_name);

                by_name.then(by_subgraph)
            })
            .map(|i| &self.endpoints[i])
            .ok()
    }
}

impl Resolver for RestExtension {
    fn resolve_field(
        &mut self,
        _: SharedContext,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        _: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let rest: Rest<'_> = directive
            .arguments()
            .map_err(|e| format!("Could not parse directive arguments: {e}"))?;

        let Some(endpoint) = self.get_endpoint(rest.endpoint, subgraph_name) else {
            return Err(format!("Endpoint not found: {}", rest.endpoint).into());
        };

        let mut url = Url::parse(&endpoint.args.base_url).map_err(|e| format!("Could not parse URL: {e}"))?;

        let path = rest.path.strip_prefix("/").unwrap_or(rest.path);

        if !path.is_empty() {
            let mut path_segments = url.path_segments_mut().map_err(|_| "Could not parse URL")?;

            path_segments.push(path);
        }

        let url = url.join(path).map_err(|e| format!("Could not parse URL path: {e}"))?;

        let builder = HttpRequest::builder(url, rest.method.into());

        let request = match rest.body() {
            Some(ref body) => builder.json(body),
            None => builder.build(),
        };

        let result = http::execute(&request).map_err(|e| format!("HTTP request failed: {e}"))?;

        if !result.status().is_success() {
            return Err(format!("HTTP request failed with status: {}", result.status()).into());
        }

        let data: serde_json::Value = result
            .json()
            .map_err(|e| format!("Error deserializing response: {e}"))?;

        let mut results = FieldOutput::new();

        if !(data.is_object() || data.is_array()) {
            results.push_value(data);
            return Ok(results);
        }

        let filtered = self
            .jq_selection
            .select(rest.selection, data)
            .map_err(|e| format!("Error selecting result value: {e}"))?;

        for result in filtered {
            match result {
                Ok(result) => results.push_value(result),
                Err(e) => results.push_error(format!("Error parsing result value: {e}")),
            }
        }

        Ok(results)
    }

    fn resolve_subscription(
        &mut self,
        _: SharedContext,
        _: &str,
        _: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        unreachable!()
    }
}
