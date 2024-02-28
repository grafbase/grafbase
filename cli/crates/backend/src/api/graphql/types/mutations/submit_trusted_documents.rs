use super::schema;

#[derive(Debug, cynic::InputObject)]
pub struct TrustedDocumentInput {
    pub document_id: String,
    pub document_text: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct TrustedDocumentsSubmitSuccess {
    pub count: i32,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ReusedId {
    pub document_id: String,
    pub existing_document_text: String,
    pub new_document_text: String,
}

#[derive(Debug, cynic::QueryFragment)]
pub struct ReusedIds {
    pub reused: Vec<ReusedId>,
}

#[derive(Debug, cynic::InlineFragments)]
pub enum TrustedDocumentsSubmitPayload {
    TrustedDocumentsSubmitSuccess(TrustedDocumentsSubmitSuccess),
    ReusedIds(ReusedIds),
    #[cynic(fallback)]
    Unknown,
}

#[derive(Debug, cynic::QueryVariables)]
pub struct TrustedDocumentsSubmitVariables<'a> {
    pub account: &'a str,
    pub project: &'a str,
    pub branch: &'a str,
    pub client_name: &'a str,
    pub documents: Vec<TrustedDocumentInput>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "TrustedDocumentsSubmitVariables")]
pub(crate) struct TrustedDocumentsSubmit {
    #[arguments(clientName : $client_name, projectSlug: $project, accountSlug : $account, branchSlug: $branch, documents: $documents)]
    pub(crate) trusted_documents_submit: TrustedDocumentsSubmitPayload,
}
