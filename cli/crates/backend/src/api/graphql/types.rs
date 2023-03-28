#[cynic::schema_for_derives(file = r#"src/api/graphql/api.graphql"#, module = "schema")]
pub mod mutations {
    use super::schema;

    #[derive(cynic::InputObject, Clone, Debug)]
    pub struct ProjectCreateInput {
        pub account_id: cynic::Id,
        pub project_slug: String,
        pub database_regions: Vec<String>,
    }

    #[derive(cynic::QueryVariables)]
    pub struct ProjectCreateArguments {
        pub input: ProjectCreateInput,
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

    #[derive(cynic::InlineFragments, Debug)]
    pub enum ProjectCreatePayload {
        ProjectCreateSuccess(ProjectCreateSuccess),
        SlugAlreadyExistsError(SlugAlreadyExistsError),
        SlugInvalidError(SlugInvalidError),
        AccountDoesNotExistError(AccountDoesNotExistError),
        SlugTooLongError(SlugTooLongError),
        CurrentPlanLimitReachedError(CurrentPlanLimitReachedError),
        EmptyDatabaseRegionsError(EmptyDatabaseRegionsError),
        DuplicateDatabaseRegionsError(DuplicateDatabaseRegionsError),
        InvalidDatabaseRegionsError(InvalidDatabaseRegionsError),
        #[cynic(fallback)]
        Unknown,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct ProjectDoesNotExistError {
        pub __typename: String,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(rename_all = "camelCase")]
    pub struct DeploymentCreateInput {
        pub archive_file_size: i32,
        pub branch: Option<String>,
        pub project_id: cynic::Id,
    }

    #[derive(cynic::QueryVariables)]
    pub struct DeploymentCreateArguments {
        pub input: DeploymentCreateInput,
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
        Unknown,
    }
}

#[cynic::schema_for_derives(file = r#"src/api/graphql/api.graphql"#, module = "schema")]
pub mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query")]
    pub struct ViewerAndRegions {
        pub viewer: Option<User>,
        pub closest_database_region: Option<DatabaseRegion>,
        pub database_regions: Vec<DatabaseRegion>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct DatabaseRegion {
        pub name: String,
        pub city: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct User {
        pub personal_account: Option<PersonalAccount>,
        pub organization_memberships: Vec<Member>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PersonalAccount {
        pub id: cynic::Id,
        pub name: String,
        pub slug: String,
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
    }
}

#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!(r#"src/api/graphql/api.graphql"#);
}
