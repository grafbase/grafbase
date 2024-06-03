use cynic::{http::ReqwestExt, QueryBuilder};

use crate::api::graphql::queries::log_entries::LogEventsQuery;

pub use super::graphql::queries::log_entries::{FunctionLogEvent, GatewayRequestLogEvent, LogEvent};
use super::graphql::queries::project_slug_by_id::GraphSlugById;
pub use super::utils::project_linked;
use super::{
    client::create_client,
    consts::api_url,
    errors::ApiError,
    graphql::queries::{
        branch_by_domain::{Account, Branch, BranchByDomain, BranchByDomainArguments, Graph},
        log_entries::{LogEventFilter, LogEventsArguments},
        project_slug_by_id::GraphSlugByIdArguments,
        viewer_for_link::{PersonalAccount, Viewer},
    },
};

/// # Errors
///
/// see [`ApiError`]
pub async fn personal_account_slug() -> Result<String, ApiError> {
    let client = create_client().await?;

    let query = Viewer::build(());

    let response = client.post(api_url()).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    let viewer_response = response.viewer.ok_or(ApiError::UnauthorizedOrDeletedUser)?;

    let PersonalAccount { slug, .. } = viewer_response
        .personal_account
        .ok_or(ApiError::IncorrectlyScopedToken)?;

    Ok(slug)
}

/// # Errors
///
/// see [`ApiError`]
pub async fn branch_by_domain(domain: &str) -> Result<Option<(String, String, String)>, ApiError> {
    let client = create_client().await?;

    let query = BranchByDomain::build(BranchByDomainArguments { domain });

    let response = client.post(api_url()).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    Ok(response.branch_by_domain.map(
        |Branch {
             graph:
                 Graph {
                     account: Account { slug: account_slug },
                     slug: project_slug,
                 },
             name,
         }| { (account_slug, project_slug, name) },
    ))
}

#[derive(Clone, Copy)]
pub enum LogEventsRange {
    Last(u16),
    After(ulid::Ulid),
}

/// # Errors
///
/// see [`ApiError`]
pub async fn logs_events_by_time_range(
    account_slug: &str,
    graph_slug: &str,
    branch: Option<&str>,
    range: LogEventsRange,
) -> Result<Vec<LogEvent>, ApiError> {
    const PAGE_SIZE: u16 = 100;

    let client = create_client().await?;

    let filter = LogEventFilter {
        branch,
        ..Default::default()
    };
    let (mut arguments, reverse) = match range {
        LogEventsRange::Last(count) => (
            LogEventsArguments {
                account_slug,
                graph_slug,
                last: Some(i32::from(count)),
                filter,
                ..Default::default()
            },
            true,
        ),
        LogEventsRange::After(after) => (
            LogEventsArguments {
                account_slug,
                graph_slug,
                after: Some(after.to_string()),
                first: Some(i32::from(PAGE_SIZE)),
                filter,
                ..Default::default()
            },
            false,
        ),
    };

    let mut has_more_pages = true;
    let mut log_events = vec![];
    while has_more_pages {
        let query = LogEventsQuery::build(arguments.clone());
        let response = client.post(api_url()).run_graphql(query).await?;
        let response = response.data.expect("must exist");

        let mut project = response.graph_by_account_slug.ok_or(ApiError::ProjectDoesNotExist)?;

        assert!(project.log_events.page_info.end_cursor >= project.log_events.page_info.start_cursor);
        if project.log_events.page_info.has_next_page {
            arguments.after = project.log_events.page_info.end_cursor.take();
            has_more_pages = arguments.after.is_some();
        } else if project.log_events.page_info.has_previous_page {
            arguments.before = project.log_events.page_info.start_cursor.take();
            let last = arguments.last.as_mut().unwrap();
            *last -= i32::try_from(project.log_events.nodes.len()).expect("must be valid");
            assert!(*last >= 0);
            has_more_pages = arguments.before.is_some();
        } else {
            has_more_pages = false;
        }
        let mut nodes = project.log_events.nodes;
        if reverse {
            nodes.reverse();
        }
        log_events.extend(nodes);
    }

    if reverse {
        log_events.reverse();
    }

    Ok(log_events)
}

/// # Errors
///
/// see [`ApiError`]
pub async fn graph_slug_by_id(id: &str) -> Result<Option<(String, String)>, ApiError> {
    let client = create_client().await?;

    let query = GraphSlugById::build(GraphSlugByIdArguments { id });

    let response = client.post(api_url()).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    Ok(response.node.and_then(|node| match node {
        super::graphql::queries::project_slug_by_id::Node::Graph(graph) => Some((graph.account.slug, graph.slug)),
        super::graphql::queries::project_slug_by_id::Node::Project(project) => {
            Some((project.account_slug, project.slug))
        }
        super::graphql::queries::project_slug_by_id::Node::Unknown => None,
    }))
}
