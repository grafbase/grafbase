use super::client::create_client;
use super::errors::{ApiError, CreateError};
use super::graphql::mutations::{
    CurrentPlanLimitReachedError, GraphCreate, GraphCreateArguments, GraphCreateInput, GraphCreatePayload,
    SlugTooLongError,
};
use super::graphql::queries::viewer_for_create::Viewer;
use super::types::Account;
use crate::common::environment::PlatformData;
use cynic::Id;
use cynic::http::ReqwestExt;
use cynic::{MutationBuilder, QueryBuilder};

/// # Errors
///
/// See [`ApiError`]
pub async fn get_viewer_data_for_creation() -> Result<Vec<Account>, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;
    let query = Viewer::build(());
    let response = client.post(&platform_data.api_url).run_graphql(query).await?;
    let response = response.data.expect("must exist");
    let viewer_response = response.viewer.ok_or(ApiError::UnauthorizedOrDeletedUser)?;

    let accounts = viewer_response
        .organizations
        .nodes
        .iter()
        .map(|organization| Account {
            id: organization.id.inner().to_owned(),
            name: organization.name.clone(),
            slug: organization.slug.clone(),
        })
        .collect();

    Ok(accounts)
}

/// # Errors
///
/// See [`ApiError`]
pub async fn create(account_id: &str, graph_slug: &str) -> Result<(Vec<String>, String), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;

    let operation = GraphCreate::build(GraphCreateArguments {
        input: GraphCreateInput {
            account_id: Id::new(account_id),
            graph_slug,
        },
    });

    let response = client.post(&platform_data.api_url).run_graphql(operation).await?;
    let payload = response.data.ok_or(ApiError::UnauthorizedOrDeletedUser)?.graph_create;

    match payload {
        GraphCreatePayload::GraphCreateSuccess(graph_create_success) => {
            let domains = graph_create_success
                .graph
                .production_branch
                .domains
                .iter()
                .map(|domain| format!("{domain}/graphql"))
                .collect();

            Ok((domains, graph_create_success.graph.slug))
        }
        GraphCreatePayload::SlugAlreadyExistsError(_) => Err(CreateError::SlugAlreadyExists.into()),
        GraphCreatePayload::SlugInvalidError(_) => Err(CreateError::SlugInvalid.into()),
        GraphCreatePayload::SlugTooLongError(SlugTooLongError { max_length, .. }) => {
            Err(CreateError::SlugTooLong { max_length }.into())
        }
        GraphCreatePayload::AccountDoesNotExistError(_) => Err(CreateError::AccountDoesNotExist.into()),
        GraphCreatePayload::CurrentPlanLimitReachedError(CurrentPlanLimitReachedError { max, .. }) => {
            Err(CreateError::CurrentPlanLimitReached { max }.into())
        }
        GraphCreatePayload::DisabledAccountError(_) => Err(CreateError::DisabledAccount.into()),
        GraphCreatePayload::Unknown(error) => Err(CreateError::Unknown(error).into()),
    }
}
