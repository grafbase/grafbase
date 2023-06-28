pub use async_graphql_parser::Error as GraphQLParseError;

#[derive(thiserror::Error, Debug)]
pub enum GeneratorError {
    #[error("Error parsing GraphQL document:\nCaused by: {0}")]
    GraphQLParse(GraphQLParseError),
    #[error("Error generating TypeScript document: {0}")]
    TypeScriptGenerate(String),
}
