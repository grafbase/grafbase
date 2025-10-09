use super::super::schema;

#[derive(cynic::InputObject, Clone, Debug)]
pub struct SchemaProposalCreateInput<'a> {
    pub title: &'a str,
    pub branch_id: cynic::Id,
    pub description: Option<&'a str>,
}

#[derive(cynic::QueryVariables)]
pub struct SchemaProposalCreateArguments<'a> {
    pub input: SchemaProposalCreateInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "SchemaProposalCreateArguments")]
pub struct SchemaProposalCreateMutation {
    #[arguments(input: $input)]
    pub schema_proposal_create: SchemaProposalCreatePayload,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaProposalCreateSuccess {
    pub schema_proposal: SchemaProposal,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaProposal {
    pub id: cynic::Id,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum SchemaProposalCreatePayload {
    SchemaProposalCreateSuccess(SchemaProposalCreateSuccess),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::InputObject, Clone, Debug)]
pub struct SchemaProposalEditSubgraph<'a> {
    pub name: &'a str,
    pub schema: Option<&'a str>,
}

#[derive(cynic::InputObject, Clone, Debug)]
pub struct SchemaProposalEditInput<'a> {
    pub schema_proposal_id: cynic::Id,
    pub subgraphs: Vec<SchemaProposalEditSubgraph<'a>>,
    pub description: Option<&'a str>,
}

#[derive(cynic::QueryVariables)]
pub struct SchemaProposalEditArguments<'a> {
    pub input: SchemaProposalEditInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "SchemaProposalEditArguments")]
pub struct SchemaProposalEditMutation {
    #[arguments(input: $input)]
    pub schema_proposal_edit: SchemaProposalEditPayload,
}

#[derive(cynic::InlineFragments, Debug)]
#[expect(unused)]
pub enum SchemaProposalEditPayload {
    SchemaProposalEditSuccess(SchemaProposalEditSuccess),
    SchemaProposalDoesNotExistError(SchemaProposalDoesNotExistError),
    SchemaProposalEditParserErrors(SchemaProposalEditParserErrors),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaProposalEditSuccess {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaProposalDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaProposalEditParserErrors {
    pub errors: Vec<SchemaProposalEditParserError>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaProposalEditParserError {
    pub subgraph_name: String,
    pub error: String,
    pub span_start: i32,
    pub span_end: i32,
}
