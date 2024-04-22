use common::{consts::PROJECT_METADATA_FILE, environment::Project};
use cynic::{http::ReqwestExt, MutationBuilder, QueryBuilder};
use tokio::fs::read_to_string;

use super::{
    client::create_client,
    consts::api_url,
    errors::{ApiError, BranchError},
    graphql::{
        mutations::{BranchDelete, BranchDeleteArguments, BranchDeletePayload},
        queries::list_branches::{ListBranches, ListBranchesArguments, Node},
    },
    types::ProjectMetadata,
};

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

pub async fn list() -> Result<Vec<(String, bool)>, ApiError> {
    let project = Project::get();

    let project_metadata_file_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

    match project_metadata_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(ApiError::UnlinkedProject),
        Err(error) => return Err(ApiError::ReadProjectMetadataFile(error)),
    }

    let project_metadata_file = read_to_string(project_metadata_file_path)
        .await
        .map_err(ApiError::ReadProjectMetadataFile)?;

    let project_metadata: ProjectMetadata =
        serde_json::from_str(&project_metadata_file).map_err(|_| ApiError::CorruptProjectMetadataFile)?;

    let operation = ListBranches::build(ListBranchesArguments {
        project_id: project_metadata.project_id.into(),
    });

    let client = create_client().await?;
    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    match (data.and_then(|d| d.node), errors) {
        (Some(Node::Project(project)), _) => {
            let project_ref = format!("{}/{}", project.account_slug, project.slug);

            let branches = project
                .branches
                .edges
                .into_iter()
                .map(|edge| {
                    (
                        format!("{}@{}", project_ref, edge.node.name),
                        edge.node.name == project.production_branch.name,
                    )
                })
                .collect();

            Ok(branches)
        }
        (_, errors) => Err(ApiError::RequestError(format!("{errors:#?}"))),
    }
}
