pub mod mutations {
    use super::schema;

    #[derive(cynic::InputObject, Clone, Debug)]
    pub struct ProjectCreateInput<'a> {
        pub account_id: cynic::Id,
        pub project_slug: &'a str,
        pub database_regions: &'a [String],
    }

    #[derive(cynic::QueryVariables)]
    pub struct ProjectCreateArguments<'a> {
        pub input: ProjectCreateInput<'a>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct SlugTooLongError {
        pub __typename: String,
        pub max_length: i32,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct SlugInvalidError {
        pub __typename: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct SlugAlreadyExistsError {
        pub __typename: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct ProjectCreateSuccess {
        pub __typename: String,
        pub project: Project,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Project {
        pub id: cynic::Id,
        pub slug: String,
        pub production_branch: Branch,
        #[arguments(last: 5)]
        pub api_keys: ProjectApiKeyConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct ProjectApiKeyConnection {
        pub nodes: Vec<ProjectApiKey>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct ProjectApiKey {
        pub key: String,
        pub name: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Mutation", variables = "ProjectCreateArguments")]
    pub struct ProjectCreate {
        #[arguments(input: $input)]
        pub project_create: ProjectCreatePayload,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct InvalidDatabaseRegionsError {
        pub __typename: String,
        pub invalid: Vec<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct EmptyDatabaseRegionsError {
        pub __typename: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DuplicateDatabaseRegionsError {
        pub __typename: String,
        pub duplicates: Vec<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct CurrentPlanLimitReachedError {
        pub __typename: String,
        pub max: i32,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Branch {
        pub domains: Vec<String>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct AccountDoesNotExistError {
        pub __typename: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DisabledAccountError {
        pub __typename: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct EnvironmentVariableCountLimitExceededError {
        pub __typename: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct InvalidEnvironmentVariablesError {
        pub __typename: String,
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum ProjectCreatePayload {
        ProjectCreateSuccess(ProjectCreateSuccess),
        SlugAlreadyExistsError(SlugAlreadyExistsError),
        DisabledAccountError(DisabledAccountError),
        SlugInvalidError(SlugInvalidError),
        SlugTooLongError(SlugTooLongError),
        AccountDoesNotExistError(AccountDoesNotExistError),
        CurrentPlanLimitReachedError(CurrentPlanLimitReachedError),
        InvalidEnvironmentVariablesError(InvalidEnvironmentVariablesError),
        EnvironmentVariableCountLimitExceededError(EnvironmentVariableCountLimitExceededError),
        InvalidDatabaseRegionsError(InvalidDatabaseRegionsError),
        DuplicateDatabaseRegionsError(DuplicateDatabaseRegionsError),
        EmptyDatabaseRegionsError(EmptyDatabaseRegionsError),
        #[cynic(fallback)]
        Unknown(String),
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct ProjectDoesNotExistError {
        pub __typename: String,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(rename_all = "camelCase")]
    pub struct DeploymentCreateInput<'a> {
        pub archive_file_size: i32,
        pub branch: Option<&'a str>,
        pub project_id: cynic::Id,
    }

    #[derive(cynic::QueryVariables)]
    pub struct DeploymentCreateArguments<'a> {
        pub input: DeploymentCreateInput<'a>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Mutation", variables = "DeploymentCreateArguments")]
    pub struct DeploymentCreate {
        #[arguments(input: $input)]
        pub deployment_create: DeploymentCreatePayload,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DeploymentCreateSuccess {
        pub __typename: String,
        pub presigned_url: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DailyDeploymentCountLimitExceededError {
        pub __typename: String,
        pub limit: i32,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct ArchiveFileSizeLimitExceededError {
        pub __typename: String,
        pub limit: i32,
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum DeploymentCreatePayload {
        DeploymentCreateSuccess(DeploymentCreateSuccess),
        ProjectDoesNotExistError(ProjectDoesNotExistError),
        ArchiveFileSizeLimitExceededError(ArchiveFileSizeLimitExceededError),
        DailyDeploymentCountLimitExceededError(DailyDeploymentCountLimitExceededError),
        #[cynic(fallback)]
        Unknown(String),
    }
}

cynic::impl_scalar!(chrono::DateTime<chrono::Utc>, schema::DateTime);

pub mod queries {
    pub mod log_entries {
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
            pub project_slug: &'a str,
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
        #[cynic(graphql_type = "Project", variables = "LogEventsArguments")]
        pub struct ProjectWithLogEvents {
            #[arguments(first: $first, after: $after, last: $last, before: $before, filter: $filter)]
            pub log_events: LogEventConnection,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query", variables = "LogEventsArguments")]
        pub struct LogEventsQuery {
            #[arguments(accountSlug: $account_slug, projectSlug: $project_slug)]
            pub project_by_account_slug: Option<ProjectWithLogEvents>,
        }
    }

    pub mod branch_by_domain {
        use super::super::schema;

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Project {
            pub slug: String,
            pub account_slug: String,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Branch {
            pub name: String,
            pub project: Project,
        }

        #[derive(cynic::QueryVariables)]

        pub struct BranchByDomainArguments<'a> {
            pub domain: &'a str,
        }

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query", variables = "BranchByDomainArguments")]
        pub struct BranchByDomain {
            #[arguments(domain: $domain)]
            pub branch_by_domain: Option<Branch>,
        }
    }

    pub mod viewer_for_create {
        use super::super::schema;

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct Viewer {
            pub viewer: Option<User>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct User {
            pub personal_account: Option<PersonalAccount>,
            #[arguments(last: 100)]
            pub organizations: OrganizationConnection,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct OrganizationConnection {
            pub nodes: Vec<Organization>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct PersonalAccount {
            pub id: cynic::Id,
            pub name: String,
            pub slug: String,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Organization {
            pub id: cynic::Id,
            pub name: String,
            pub slug: String,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Account {
            pub id: cynic::Id,
            pub name: String,
            pub slug: String,
        }
    }

    pub mod viewer_for_link {
        use super::super::schema;

        #[derive(cynic::QueryFragment, Debug)]
        #[cynic(graphql_type = "Query")]
        pub struct Viewer {
            pub viewer: Option<User>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct ProjectConnection {
            pub nodes: Vec<Project>,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct OrganizationConnection {
            pub nodes: Vec<Organization>,
        }

        #[derive(cynic::QueryFragment, Debug, Clone)]
        pub struct Project {
            pub id: cynic::Id,
            pub slug: String,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct User {
            pub personal_account: Option<PersonalAccount>,
            #[arguments(last: 100)]
            pub organizations: OrganizationConnection,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct PersonalAccount {
            pub id: cynic::Id,
            pub name: String,
            pub slug: String,
            #[arguments(last: 100)]
            pub projects: ProjectConnection,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Member {
            pub account: Account,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Account {
            pub id: cynic::Id,
            pub name: String,
            pub slug: String,
            #[arguments(last: 100)]
            pub projects: ProjectConnection,
        }

        #[derive(cynic::QueryFragment, Debug)]
        pub struct Organization {
            pub id: cynic::Id,
            pub name: String,
            pub slug: String,
            #[arguments(last: 100)]
            pub projects: ProjectConnection,
        }
    }
}

#[cynic::schema("grafbase")]
mod schema {}
