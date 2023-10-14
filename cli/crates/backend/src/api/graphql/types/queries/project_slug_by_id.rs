use super::super::schema;

#[derive(Clone, Default, cynic::QueryVariables)]
pub struct ProjectSlugByIdArguments<'a> {
    pub id: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Project", variables = "ProjectSlugByIdArguments")]
pub struct ProjectSlugByIdProject {
    pub account_slug: String,
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ProjectSlugByIdArguments")]
pub struct ProjectSlugById {
    #[arguments(id: $id)]
    pub project_by_id: Option<ProjectSlugByIdProject>,
}
