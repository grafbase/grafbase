use grafbase_sdk::{
    host_io::http::{self, HttpRequest, Url},
    types::{Directive, FieldDefinition, FieldInputs, FieldOutput},
    Error, Extension, Resolver, ResolverExtension, SharedContext,
};
use jaq_interpret::{Ctx, Filter, FilterT, ParseCtx, RcIter, Val};
use std::collections::HashMap;

#[derive(ResolverExtension)]
struct RestExtension {
    endpoints: Vec<RestEndpoint>,
    filters: HashMap<String, Filter>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestEndpoint {
    name: String,
    http: HttpSettings,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpSettings {
    base_url: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Rest<'a> {
    endpoint: &'a str,
    http: HttpCall<'a>,
    selection: &'a str,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpCall<'a> {
    method: HttpMethod,
    path: &'a str,
}

#[derive(serde::Deserialize)]
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
    fn new(schema_directives: Vec<Directive>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::<RestEndpoint>::new();

        for directive in schema_directives {
            endpoints.push(directive.arguments()?);
        }

        endpoints.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(Self {
            endpoints,
            filters: HashMap::new(),
        })
    }
}

impl RestExtension {
    pub fn get_endpoint(&self, endpoint: &str) -> Option<&RestEndpoint> {
        self.endpoints
            .binary_search_by(|e| e.name.as_str().cmp(endpoint))
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

        let Some(endpoint) = self.get_endpoint(rest.endpoint) else {
            return Err(Error {
                extensions: Vec::new(),
                message: format!("Endpoint not found: {}", rest.endpoint),
            });
        };

        let url = Url::parse(&format!("{}/{}", endpoint.http.base_url, rest.http.path)).map_err(|e| Error {
            extensions: Vec::new(),
            message: format!("Could not parse URL: {e}"),
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
