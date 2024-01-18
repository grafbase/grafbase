pub use super::graphql::mutations::{SchemaCheck, SchemaCheckErrorSeverity, SchemaCheckGitCommitInput};

use super::{
    client::create_client,
    consts::API_URL,
    errors::ApiError,
    graphql::mutations::{SchemaCheckCreate, SchemaCheckCreateArguments, SchemaCheckCreateInput, SchemaCheckPayload},
};
use cynic::{http::ReqwestExt, MutationBuilder};

pub async fn check(
    // The Good Code™
    account: &str,
    project: &str,
    branch: Option<&str>,
    subgraph_name: Option<&str>,
    schema: &str,
    git_commit: Option<SchemaCheckGitCommitInput>,
) -> Result<SchemaCheck, ApiError> {
    let client = create_client().await?;

    let operation = SchemaCheckCreate::build(SchemaCheckCreateArguments {
        input: SchemaCheckCreateInput {
            account_slug: account,
            project_slug: project,
            branch,
            subgraph_name,
            schema,
            git_commit,
        },
    });

    let result = client.post(API_URL).run_graphql(operation).await?;

    match result {
        cynic::GraphQlResponse {
            data:
                Some(SchemaCheckCreate {
                    schema_check_create: Some(SchemaCheckPayload::SchemaCheck(sc)),
                }),
            errors: _,
        } => Ok(sc),
        _ => Err(ApiError::RequestError(format!("API error:\n\n{result:#?}",))),
    }
}
