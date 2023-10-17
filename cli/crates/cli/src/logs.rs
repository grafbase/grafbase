use std::str::FromStr;

use backend::api::{
    errors::ApiError,
    logs::{self as api, LogEventsRange},
};
use common::types::{LogLevel, UdfKind};
use itertools::Itertools;

use crate::errors::CliError;

#[derive(Debug)]
pub struct BranchReference {
    account_slug: String,
    project_slug: String,
    branch_name: Option<String>,
}

pub async fn project_branch_reference_to_account_project_slug(
    project_branch_reference: Option<String>,
) -> Result<BranchReference, CliError> {
    Ok(if let Some(project_branch_reference) = project_branch_reference {
        let elements = project_branch_reference.split('/').collect_vec();
        match elements.as_slice() {
            [account_slug, project_slug] => BranchReference {
                account_slug: (*account_slug).to_string(),
                project_slug: (*project_slug).to_string(),
                branch_name: None,
            },
            [project_slug] if !project_slug.contains('.') => {
                let account_slug = api::personal_account_slug().await.map_err(CliError::BackendApiError)?;
                BranchReference {
                    account_slug,
                    project_slug: (*project_slug).to_string(),
                    branch_name: None,
                }
            }
            _ => {
                let domain = url::Url::from_str(&project_branch_reference)
                    .ok()
                    .and_then(|parsed_url| parsed_url.domain().map(str::to_owned))
                    .unwrap_or_else(|| project_branch_reference.clone());

                let (account_slug, project_slug, branch_name) = api::branch_by_domain(&domain)
                    .await
                    .map_err(CliError::BackendApiError)?
                    .ok_or_else(|| CliError::ProjectNotFound(project_branch_reference))?;
                BranchReference {
                    account_slug,
                    project_slug,
                    branch_name: Some(branch_name),
                }
            }
        }
    } else {
        let project_metadata = api::project_linked()
            .await
            .map_err(CliError::BackendApiError)?
            .ok_or(CliError::LogsNoLinkedProject)?;
        let (account_slug, project_slug) = api::project_slug_by_id(&project_metadata.project_id)
            .await
            .map_err(CliError::BackendApiError)?
            .ok_or_else(|| CliError::ProjectNotFound(project_metadata.project_id))?;
        BranchReference {
            account_slug,
            project_slug,
            branch_name: None,
        }
    })
}

pub enum LogEventType {
    Request {
        http_method: String,
        path: String,
        http_status: u16,
        duration: std::time::Duration,
    },
    FunctionMessage {
        log_level: LogLevel,
        function_kind: UdfKind,
        function_name: String,
    },
}

pub struct LogEvent {
    pub id: ulid::Ulid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub log_event_type: LogEventType,
}

pub async fn log_events_for_time_range(
    account_slug: &str,
    project_slug: &str,
    branch: Option<&str>,
    range: LogEventsRange,
) -> Result<Vec<LogEvent>, CliError> {
    let mut log_events: Vec<_> = api::logs_events_by_time_range(account_slug, project_slug, branch, range)
        .await
        .map_err(|err| match err {
            ApiError::ProjectDoesNotExist => CliError::ProjectNotFound(format!("{account_slug}/{project_slug}")),
            other => CliError::BackendApiError(other),
        })?
        .into_iter()
        .filter_map(|log_event| match log_event {
            api::LogEvent::GatewayRequestLogEvent(api::GatewayRequestLogEvent {
                id,
                created_at,
                http_method,
                http_status,
                url,
                message,
                duration,
                ..
            }) => Some(LogEvent {
                id: id.parse().expect("must be a valid ULID"),
                created_at,
                message,
                log_event_type: LogEventType::Request {
                    http_method,
                    path: url::Url::from_str(&url).expect("must be a valid URL").path().to_owned(),
                    http_status: u16::try_from(http_status).expect("must be valid"),
                    duration: std::time::Duration::from_millis(duration.try_into().expect("must be a positive number")),
                },
            }),
            api::LogEvent::FunctionLogEvent(api::FunctionLogEvent {
                id,
                created_at,
                message,
                function_kind,
                function_name,
                log_level,
                ..
            }) => Some(LogEvent {
                id: id.parse().expect("must be a valid ULID"),
                created_at,
                message,
                log_event_type: LogEventType::FunctionMessage {
                    function_kind: function_kind.into(),
                    function_name,
                    log_level: log_level.into(),
                },
            }),
            _ => None,
        })
        .collect();

    let log_event_count = log_events.len();
    log_events.dedup_by_key(|log_event| log_event.id);
    assert_eq!(log_events.len(), log_event_count);

    Ok(log_events)
}

#[tokio::main]
pub async fn logs(project_branch_reference: Option<String>, limit: u16, follow: bool) -> Result<(), CliError> {
    let project_branch_reference = project_branch_reference_to_account_project_slug(project_branch_reference).await?;

    let mut range = LogEventsRange::Last(limit);

    loop {
        let new_events = log_events_for_time_range(
            &project_branch_reference.account_slug,
            &project_branch_reference.project_slug,
            project_branch_reference.branch_name.as_deref(),
            range,
        )
        .await?;

        if matches!(range, LogEventsRange::Last(_)) || !new_events.is_empty() {
            let last_id = new_events.last().as_ref().map_or_else(
                || {
                    ulid::Ulid::from_parts(
                        chrono::Utc::now().timestamp_millis().try_into().expect("must be valid"),
                        0,
                    )
                },
                |event| event.id,
            );
            range = LogEventsRange::After(last_id);
        }
        for new_event in new_events {
            crate::output::report::print_log_entry(new_event);
        }

        if !follow {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
