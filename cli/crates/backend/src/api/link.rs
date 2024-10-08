use super::{
    client::create_client,
    consts::api_url,
    errors::ApiError,
    graphql::queries::viewer_for_link::{PersonalAccount, Viewer},
    types::{self, AccountWithGraphs, ProjectMetadata},
    utils::has_project_linked,
};
use common::consts::PROJECT_METADATA_FILE;
use common::environment::Project;
use cynic::{http::ReqwestExt, QueryBuilder};
use std::iter;

/// # Errors
///
/// see [`ApiError`]
pub async fn project_link_validations() -> Result<(), ApiError> {
    if has_project_linked().await? {
        Err(ApiError::ProjectAlreadyLinked)
    } else {
        Ok(())
    }
}

/// # Errors
///
/// see [`ApiError`]
#[allow(clippy::module_name_repetitions)]
pub async fn get_viewer_data_for_link() -> Result<Vec<AccountWithGraphs>, ApiError> {
    let client = create_client().await?;
    let query = Viewer::build(());
    let response = client.post(api_url()).run_graphql(query).await?;
    let response = response.data.expect("must exist");
    let viewer_response = response.viewer.ok_or(ApiError::UnauthorizedOrDeletedUser)?;

    let PersonalAccount { id, name, slug, graphs } = viewer_response
        .personal_account
        .ok_or(ApiError::IncorrectlyScopedToken)?;

    let personal_account_id = id;

    let personal_account = AccountWithGraphs {
        id: personal_account_id.inner().to_owned(),
        name,
        slug,
        personal: true,
        graphs: graphs
            .nodes
            .into_iter()
            .map(|project| types::Graph {
                id: project.id.into_inner(),
                slug: project.slug,
            })
            .collect(),
    };

    let accounts = iter::once(personal_account)
        .chain(viewer_response.organizations.nodes.iter().map(|organization| {
            AccountWithGraphs {
                id: organization.id.inner().to_owned(),
                name: organization.name.clone(),
                slug: organization.slug.clone(),
                personal: false,
                graphs: organization
                    .graphs
                    .nodes
                    .iter()
                    .cloned()
                    .map(|project| types::Graph {
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
pub async fn link_project(project_id: String) -> Result<(), ApiError> {
    let project = Project::get();
    match project.dot_grafbase_directory_path.try_exists() {
        Ok(true) => {}
        Ok(false) => tokio::fs::create_dir_all(&project.dot_grafbase_directory_path)
            .await
            .map_err(ApiError::CreateProjectDotGrafbaseFolder)?,
        Err(error) => return Err(ApiError::ReadProjectDotGrafbaseFolder(error)),
    }
    let project_metadata_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);
    tokio::fs::write(&project_metadata_path, ProjectMetadata::new(project_id).to_string())
        .await
        .map_err(ApiError::WriteProjectMetadataFile)
}
