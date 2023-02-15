use graph::OpenApiGraph;
use openapiv3::OpenAPI;
use parsing::components::Ref;

mod graph;
mod output;
mod parsing;

pub fn parse_spec(data: &str) -> String {
    let spec = serde_json::from_str::<OpenAPI>(data).unwrap();

    let graph = OpenApiGraph::new(parsing::parse(spec).unwrap());

    output::output(&graph).unwrap()
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The schema component {0} was a reference, which we don't currently support.")]
    TopLevelSchemaWasReference(String),
    #[error("The path component {0} was a reference, which we don't currently support.")]
    TopLevelPathWasReference(String),
    #[error("The response component {0} was a reference, which we don't currently support.")]
    TopLevelResponseWasReference(String),
    #[error("The request body component {0} was a reference, which we don't currently support.")]
    TopLevelRequestBodyWasReference(String),
    #[error("Couldn't parse HTTP verb: {0}")]
    UnknownHttpVerb(String),
    #[error("The operation {0} didn't have a response schema")]
    OperationMissingResponseSchema(String),
    #[error("Encountered an array without items, which we don't currently support")]
    ArrayWithoutItems,
    #[error("Encountered a not schema, which we don't currently support")]
    NotSchema,
    #[error("Encountered an allOf schema, which we don't currently support")]
    AllOfSchema,
    #[error("Encountered an any schema, which we don't currently support")]
    AnySchema,
    #[error("Found a reference {0} which didn't seem to exist in the spec")]
    UnresolvedReference(Ref),
}

fn is_ok(status: &openapiv3::StatusCode) -> bool {
    match status {
        openapiv3::StatusCode::Code(200) => true,
        openapiv3::StatusCode::Range(_range) => todo!(),
        _ => false,
    }
}
