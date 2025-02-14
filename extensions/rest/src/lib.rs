mod types;

use grafbase_sdk::{
    host_io::http::{self, HttpRequest, Url},
    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
    Error, Extension, Resolver, ResolverExtension, SharedContext,
};
use jaq_core::{
    load::{Arena, File, Loader},
    Compiler, Ctx, Filter, Native, RcIter,
};
use jaq_json::Val;
use std::collections::HashMap;
use types::{Rest, RestEndpoint};

#[derive(ResolverExtension)]
struct RestExtension {
    endpoints: Vec<RestEndpoint>,
    filters: HashMap<String, Filter<Native<Val>>>,
    arena: Arena,
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

        let arena = Arena::default();

        Ok(Self {
            endpoints,
            filters: HashMap::new(),
            arena,
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

    pub fn create_filter<'a>(&'a mut self, selection: &str) -> Result<&'a Filter<Native<Val>>, Error> {
        if !self.filters.contains_key(selection) {
            let program = File {
                code: selection,
                path: (),
            };

            let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));

            let modules = loader.load(&self.arena, program).map_err(|e| {
                let error = e.first().map(|e| e.0.code).unwrap_or_default();

                Error {
                    extensions: Vec::new(),
                    message: format!("The selection is not valid jq syntax: `{error}`"),
                }
            })?;

            let filter = Compiler::default()
                .with_funs(jaq_std::funs().chain(jaq_json::funs()))
                .compile(modules)
                .map_err(|e| {
                    let error = e.first().map(|e| e.0.code).unwrap_or_default();

                    Error {
                        extensions: Vec::new(),
                        message: format!("The selection is not valid jq syntax: `{error}`"),
                    }
                })?;

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

        let mut url = Url::parse(&endpoint.args.base_url).map_err(|e| Error {
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

        let builder = HttpRequest::builder(url, rest.http.method.into());

        let request = match rest.body() {
            Some(ref body) => builder.json(body),
            None => builder.build(),
        };

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
