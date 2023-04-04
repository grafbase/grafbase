use super::{
    client::create_client,
    consts::{API_URL, PROJECT_METADATA_FILE},
    errors::ApiError,
    graphql::queries::viewer::{PersonalAccount, Viewer},
    types::{AccountWithProjects, Project, ProjectMetadata},
    utils::project_linked,
};
use common::environment::Environment;
use cynic::{http::ReqwestExt, QueryBuilder};
use std::iter;

/// # Errors
///
/// see [`ApiError`]
#[allow(clippy::module_name_repetitions)]
pub async fn get_viewer_data_for_link() -> Result<Vec<AccountWithProjects>, ApiError> {
    // TODO consider if we want to do this elsewhere
    if project_linked().await? {
        return Err(ApiError::ProjectAlreadyLinked);
    }

    let client = create_client().await?;

    let query = Viewer::build(());

    let response = client.post(API_URL).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    let viewer_response = response.viewer.ok_or(ApiError::UnauthorizedOrDeletedUser)?;

    let PersonalAccount {
        id,
        name,
        slug,
        projects,
    } = viewer_response
        .personal_account
        .ok_or(ApiError::IncorrectlyScopedToken)?;

    let personal_account_id = id;

    let personal_account = AccountWithProjects {
        id: personal_account_id.inner().to_owned(),
        name,
        slug,
        personal: true,
        projects: projects
            .nodes
            .into_iter()
            .map(|project| Project {
                id: project.id.into_inner(),
                slug: project.slug,
            })
            .collect(),
    };

    let accounts = iter::once(personal_account)
        .chain(viewer_response.organizations.nodes.iter().map(|organization| {
            AccountWithProjects {
                id: organization.id.inner().to_owned(),
                name: organization.name.clone(),
                slug: organization.slug.clone(),
                personal: false,
                projects: organization
                    .projects
                    .nodes
                    .iter()
                    .cloned()
                    .map(|project| Project {
                        id: project.id.into_inner(),
                        slug: project.slug,
                    })
                    .collect(),
            }
        }))
        .collect();

    Ok(accounts)
}

/// # Errors
///
/// see [`ApiError`]
#[allow(clippy::module_name_repetitions)]
pub async fn link_project(account_id: String, project_id: String) -> Result<(), ApiError> {
    let environment = Environment::get();
    let project_metadata_path = environment.project_dot_grafbase_path.join(PROJECT_METADATA_FILE);
    tokio::fs::write(
        &project_metadata_path,
        ProjectMetadata { account_id, project_id }.to_string(),
    )
    .await
    .map_err(ApiError::WriteProjectMetadataFile)
}
