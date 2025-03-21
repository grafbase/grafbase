use super::{
    client::create_client,
    errors::{ApiError, BranchError},
    graphql::mutations::{
        BranchCreate, BranchCreateArguments, BranchCreateInput, BranchDelete, BranchDeleteArguments,
        BranchDeletePayload,
    },
};
use crate::common::environment::PlatformData;
use cynic::{MutationBuilder, http::ReqwestExt};

/// # Errors
///
/// See [`ApiError`]
pub async fn delete(account_slug: &str, graph_slug: &str, branch_name: &str) -> Result<(), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let operation = BranchDelete::build(BranchDeleteArguments {
        account_slug,
        graph_slug,
        branch_name,
    });

    let cynic::GraphQlResponse { data, errors } = client.post(platform_data.api_url()).run_graphql(operation).await?;

    if let Some(data) = data {
        match data.branch_delete {
            BranchDeletePayload::Success(_) => Ok(()),
            BranchDeletePayload::BranchDoesNotExist(_) => {
                Err(BranchError::BranchDoesNotExist(format!("{account_slug}/{graph_slug}@{branch_name}")).into())
            }
            BranchDeletePayload::CannotDeleteProductionBranch(_) => Err(BranchError::CannotDeleteProductionBranch(
                format!("{account_slug}/{graph_slug}@{branch_name}"),
            )
            .into()),
            BranchDeletePayload::Unknown(error) => Err(BranchError::Unknown(error).into()),
        }
    } else {
        Err(ApiError::RequestError(format!("{errors:#?}")))
    }
}

pub async fn create(account_slug: &str, graph_slug: &str, branch_name: &str) -> Result<(), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let operation = BranchCreate::build(BranchCreateArguments {
        input: BranchCreateInput {
            account_slug,
            graph_slug,
            branch_name,
        },
    });

    let cynic::GraphQlResponse { data, errors } = client.post(platform_data.api_url()).run_graphql(operation).await?;

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
