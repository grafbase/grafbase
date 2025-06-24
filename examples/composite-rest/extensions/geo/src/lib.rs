use grafbase_sdk::{
    ResolverExtension,
    host_io::http::{self, HttpRequest, Url},
    types::{Configuration, Error, ResolvedField, Response, SubgraphHeaders, SubgraphSchema, Variables},
};
use itertools::Itertools as _;

#[derive(ResolverExtension)]
struct Geo {
    url: Url,
    subgraph_schemas: Vec<SubgraphSchema>,
}

impl ResolverExtension for Geo {
    fn new(subgraph_schemas: Vec<SubgraphSchema>, _config: Configuration) -> Result<Self, Error> {
        Ok(Self {
            url: "https://geo.api.gouv.fr/".parse().unwrap(),
            subgraph_schemas,
        })
    }

    fn resolve(&mut self, prepared: &[u8], _headers: SubgraphHeaders, variables: Variables) -> Result<Response, Error> {
        // field which must be resolved. The prepared bytes can be customized to store anything you need in the operation cache.
        let field = ResolvedField::try_from(prepared)?;
        let schema = self
            .subgraph_schemas
            .iter()
            .find(|schema| schema.subgraph_name() == field.subgraph_name())
            .unwrap();
        let FieldArguments { code } = field.arguments(&variables)?;

        let mut request = HttpRequest::get(
            self.url
                .join(&format!("{}s/{}", field.definition(schema).name(), code))
                .unwrap(),
        );
        request
            .url()
            .query_pairs_mut()
            .append_pair(
                "fields",
                &field
                    .selection_set()
                    .fields()
                    .map(|field| field.definition(schema).name())
                    .join(","),
            )
            .append_pair("format", "json");

        let response = http::execute(request)?;

        if response.status().is_success() {
            Ok(Response::json(response.into_bytes()))
        } else if response.status().as_u16() == 404 {
            Ok(Response::null())
        } else {
            Err(Error::new("Geo API request failed"))
        }
    }
}

#[derive(serde::Deserialize)]
struct FieldArguments {
    code: String,
}
