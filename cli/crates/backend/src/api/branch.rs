use chrono::{DateTime, Utc};
use cynic::{http::ReqwestExt, MutationBuilder};

use crate::api::graphql::mutations::{BranchCreate, BranchCreateArguments, BranchCreateInput};

use super::{
    client::create_client,
    consts::api_url,
    errors::{ApiError, BranchError},
    graphql::mutations::{BranchDelete, BranchDeleteArguments, BranchDeletePayload},
};

pub struct Branch {
    pub account: String,
    pub graph: String,
    pub branch: String,
    pub is_production: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

/// # Errors
///
/// See [`ApiError`]
pub async fn delete(account_slug: &str, project_slug: &str, branch_name: &str) -> Result<(), ApiError> {
    let client = create_client().await?;

    let operation = BranchDelete::build(BranchDeleteArguments {
        account_slug,
        project_slug,
        branch_name,
    });

    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    if let Some(data) = data {
        match data.branch_delete {
            BranchDeletePayload::Success(_) => Ok(()),
            BranchDeletePayload::BranchDoesNotExist(_) => {
                Err(BranchError::BranchDoesNotExist(format!("{account_slug}/{project_slug}@{branch_name}")).into())
            }
            BranchDeletePayload::CannotDeleteProductionBranch(_) => Err(
                BranchError::CannotDeleteProductionBranchError(format!("{account_slug}/{project_slug}@{branch_name}"))
                    .into(),
            ),
            BranchDeletePayload::Unknown(error) => Err(BranchError::Unknown(error).into()),
        }
    } else {
        Err(ApiError::RequestError(format!("{errors:#?}")))
    }
}

pub async fn create(account_slug: &str, graph_slug: &str, branch_name: &str) -> Result<(), ApiError> {
    let client = create_client().await?;

    let operation = BranchCreate::build(BranchCreateArguments {
        input: BranchCreateInput {
            account_slug,
            graph_slug,
            branch_name,
        },
    });

    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    let Some(data) = data else {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    };

    match data.branch_create {
        super::graphql::mutations::BranchCreatePayload::Success(_) => Ok(()),
        super::graphql::mutations::BranchCreatePayload::BranchAlreadyExists(_) => Err(ApiError::BranchError(
            BranchError::BranchAlreadyExists(format!("{account_slug}/{graph_slug}@{branch_name}")),
        )),
        super::graphql::mutations::BranchCreatePayload::GraphDoesNotExist(_) => Err(ApiError::GraphDoesNotExist),
        super::graphql::mutations::BranchCreatePayload::GraphNotSelfHosted(_) => Err(ApiError::GraphNotSelfHosted),
        super::graphql::mutations::BranchCreatePayload::Unknown(error) => Err(BranchError::Unknown(error).into()),
    }
}
