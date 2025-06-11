use grafbase_sdk::{
    ResolverExtension,
    types::{Configuration, Error, ResolvedField, Response, SubgraphHeaders, SubgraphSchema, Variables},
};

#[derive(ResolverExtension)]
struct EchoExtension;

#[derive(serde::Deserialize)]
struct HelloArguments {
    to: String,
}

impl ResolverExtension for EchoExtension {
    fn new(_subgraph_schemas: Vec<SubgraphSchema<'_>>, _config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn resolve(
        &mut self,
        prepared: &[u8],
        _headers: SubgraphHeaders,
        _variables: Variables,
    ) -> Result<Response, Error> {
        let field = ResolvedField::try_from(prepared)?;

        let value = match field.directive().name() {
            "hello" => {
                let args: HelloArguments = field.directive().arguments().map_err(|err| err.to_string())?;
                format!("Hello, {}", args.to)
            }
            other => format!("unknown directive `{other}`"),
        };

        Ok(Response::data(serde_json::json!(value)))
    }
}
