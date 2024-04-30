use chrono::{DateTime, Utc};

use crate::api::graphql::queries::list_env_vars::BranchEnvironment;

use super::errors::ApiError;

mod create;
mod delete;
mod list;

pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
    pub environments: Vec<&'static str>,
}

pub async fn list(graph_ref: Option<(String, String)>) -> Result<Vec<EnvironmentVariable>, ApiError> {
    let project = match graph_ref {
        Some((ref account_slug, ref project_slug)) => list::with_slugs(account_slug, project_slug).await?,
        None => list::with_linked_project().await?,
    };

    let variables = project
        .environment_variables
        .edges
        .into_iter()
        .map(|edge| EnvironmentVariable {
            name: edge.node.name,
            value: edge.node.value,
            updated_at: edge.node.updated_at,
            environments: edge
                .node
                .environments
                .into_iter()
                .map(|env| match env {
                    BranchEnvironment::Preview => "preview",
                    BranchEnvironment::Production => "production",
                })
                .collect(),
        })
        .collect();

    Ok(variables)
}

pub async fn create(
    graph_ref: Option<(String, String)>,
    name: &str,
    value: &str,
    branch_environment: impl IntoIterator<Item = &str>,
) -> Result<(), ApiError> {
    match graph_ref {
        Some((account_slug, project_slug)) => {
            create::with_slugs(&account_slug, &project_slug, name, value, branch_environment).await
        }
        None => create::with_linked(name, value, branch_environment).await,
    }
}

pub async fn delete(
    graph_ref: Option<(String, String)>,
    name: &str,
    branch_environment: impl IntoIterator<Item = &str>,
) -> Result<(), ApiError> {
    match graph_ref {
        Some((account_slug, project_slug)) => {
            delete::with_slugs(&account_slug, &project_slug, name, branch_environment).await
        }
        None => delete::with_linked(name, branch_environment).await,
    }
}
