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

pub mod queries {
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
