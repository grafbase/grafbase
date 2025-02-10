use grafbase_sdk::{
    host_io::http::{self, HttpRequest, Url},
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
    Error, Extension, Resolver, ResolverExtension, SharedContext,
};
use jaq_interpret::{Ctx, Filter, FilterT, ParseCtx, RcIter, Val};
use std::collections::HashMap;

#[derive(ResolverExtension)]
struct RestExtension {
    endpoints: Vec<RestEndpoint>,
    filters: HashMap<String, Filter>,
}

#[derive(Debug)]
struct RestEndpoint {
    subgraph_name: String,
    args: RestEndpointArgs,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RestEndpointArgs {
    name: String,
    http: HttpSettings,
}

#[derive(serde::Deserialize, Debug)]
struct HttpSettings {
    #[serde(rename = "baseURL")]
    base_url: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Rest<'a> {
    endpoint: &'a str,
    http: HttpCall<'a>,
    selection: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpCall<'a> {
    method: HttpMethod,
    path: &'a str,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum HttpMethod {
    Get,
}

impl From<HttpMethod> for ::http::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Self::GET,
        }
    }
}

impl Extension for RestExtension {
    fn new(schema_directives: Vec<Directive>, _: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
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
            filters: HashMap::new(),
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

    pub fn create_filter<'a>(&'a mut self, selection: &str) -> Result<&'a Filter, Error> {
        if !self.filters.contains_key(selection) {
            let mut defs = ParseCtx::new(Vec::new());

            let (filter, errors) = jaq_parse::parse(selection, jaq_parse::main());

            if !errors.is_empty() {
                return Err(Error {
                    extensions: Vec::new(),
                    message: format!("The selection is not in valid jq syntax: {errors:?}"),
                });
            }

            let filter = defs.compile(filter.expect("we handled errors above"));

            if !defs.errs.is_empty() {
                return Err(Error {
                    extensions: Vec::new(),
                    message: "Error compiling jq filter".to_string(),
                });
            }

            self.filters.insert(selection.to_string(), filter);
        }

        Ok(self.filters.get(selection).unwrap())
    }
}

impl Resolver for RestExtension {
    fn resolve_field(
        &mut self,
        _: SharedContext,
        directive: Directive,
        _: FieldDefinition,
        _: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        let rest: Rest<'_> = directive.arguments().map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Could not parse directive arguments: {e}"),
        })?;

        let Some(endpoint) = self.get_endpoint(rest.endpoint, directive.subgraph_name()) else {
            return Err(Error {
                extensions: Vec::new(),
                message: format!("Endpoint not found: {}", rest.endpoint),
            });
        };

        let mut url = Url::parse(&endpoint.args.http.base_url).map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Could not parse URL: {e}"),
        })?;

        let path = rest.http.path.strip_prefix("/").unwrap_or(rest.http.path);

        if !path.is_empty() {
            let mut path_segments = url.path_segments_mut().map_err(|_| Error {
                extensions: Vec::new(),
                message: "Could not parse URL".to_string(),
            })?;

            path_segments.push(path);
        }

        let url = url.join(path).map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Could not parse URL path: {e}"),
        })?;

        let request = HttpRequest::builder(url, rest.http.method.into()).build();

        let result = http::execute(&request).map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("HTTP request failed: {e}"),
        })?;

        if !result.status().is_success() {
            return Err(Error {
                extensions: Vec::new(),
                message: format!("HTTP request failed with status: {}", result.status()),
            });
        }

        let data: serde_json::Value = result.json().map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Error deserializing response: {e}"),
        })?;

        let filter = self.create_filter(rest.selection)?;
        let inputs = RcIter::new(core::iter::empty());

        let filtered = filter.run((Ctx::new([], &inputs), Val::from(data)));

        let mut results = FieldOutput::new();

        for result in filtered {
            match result {
                Ok(result) => results.push_value(serde_json::Value::from(result)),
                Err(e) => results.push_error(Error {
                    extensions: Vec::new(),
                    message: format!("Error parsing result value: {e}"),
                }),
            }
        }

        Ok(results)
    }
}
