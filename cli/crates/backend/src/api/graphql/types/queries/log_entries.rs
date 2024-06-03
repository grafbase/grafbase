use common::types::UdfKind;

use super::super::schema;

#[derive(cynic::Enum, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for common::types::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => Self::Debug,
            LogLevel::Info => Self::Info,
            LogLevel::Warn => Self::Warn,
            LogLevel::Error => Self::Error,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(cynic::InlineFragments, Debug, serde::Serialize)]
pub enum LogEvent {
    GatewayRequestLogEvent(GatewayRequestLogEvent),
    FunctionLogEvent(FunctionLogEvent),
    RequestLogEvent(RequestLogEvent),
    #[cynic(fallback)]
    Other,
}

#[derive(cynic::Enum, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Clone, Copy, cynic::Enum, Debug, Eq, Hash, PartialEq)]
pub enum BranchEnvironment {
    Preview,
    Production,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct GatewayRequestLogEventOperation {
    name: Option<String>,
    #[cynic(rename = "type")]
    operation_type: OperationType,
}

#[derive(cynic::Enum, Debug, PartialEq)]
pub enum FunctionKind {
    Authorizer,
    Resolver,
}

impl From<FunctionKind> for UdfKind {
    fn from(kind: FunctionKind) -> Self {
        match kind {
            FunctionKind::Authorizer => Self::Authorizer,
            FunctionKind::Resolver => Self::Resolver,
        }
    }
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct FunctionLogEvent {
    #[serde(skip)]
    pub id: String,
    #[serde(skip)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip)]
    pub region: String,
    pub log_level: LogLevel,
    pub message: String,
    pub function_kind: FunctionKind,
    pub function_name: String,
    pub environment: BranchEnvironment,
    pub branch: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct GatewayRequestLogEvent {
    #[serde(skip)]
    pub id: String,
    #[serde(skip)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip)]
    pub region: String,
    pub log_level: LogLevel,
    pub http_method: String,
    pub http_status: i32,
    pub url: String,
    pub duration: i32,
    pub operation: Option<GatewayRequestLogEventOperation>,
    pub environment: BranchEnvironment,
    pub branch: String,
    pub message: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct RequestLogEvent {
    #[serde(skip)]
    pub id: String,
    #[serde(skip)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip)]
    pub region: String,
    pub log_level: LogLevel,
    pub http_method: String,
    pub http_status: i32,
    pub url: String,
    pub duration: i32,
    pub environment: BranchEnvironment,
    pub branch: String,
    pub message: String,
}

#[derive(Clone, Default, cynic::InputObject, Debug)]
pub struct LogEventFilter<'a> {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub branch: Option<&'a str>,
}

#[derive(Clone, Default, cynic::QueryVariables)]
pub struct LogEventsArguments<'a> {
    pub account_slug: &'a str,
    pub graph_slug: &'a str,
    pub first: Option<i32>,
    pub after: Option<String>,
    pub last: Option<i32>,
    pub before: Option<String>,
    pub filter: LogEventFilter<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct LogEventConnection {
    pub nodes: Vec<LogEvent>,
    pub page_info: PageInfo,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Graph", variables = "LogEventsArguments")]
pub struct GraphWithLogEvents {
    #[arguments(first: $first, after: $after, last: $last, before: $before, filter: $filter)]
    pub log_events: LogEventConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "LogEventsArguments")]
pub struct LogEventsQuery {
    #[arguments(accountSlug: $account_slug, graphSlug: $graph_slug)]
    pub graph_by_account_slug: Option<GraphWithLogEvents>,
}
