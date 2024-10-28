pub use crate::api::graphql::mutations::submit_trusted_documents::{
    ReusedId, ReusedIds, TrustedDocumentInput, TrustedDocumentsSubmitPayload, TrustedDocumentsSubmitVariables,
};

use super::graphql::mutations::submit_trusted_documents::TrustedDocumentsSubmit;
use crate::api::{client::create_client, errors::ApiError};
use common::environment::PlatformData;
use cynic::{http::ReqwestExt, MutationBuilder};

#[tokio::main]
pub async fn submit_trusted_documents(
    variables: TrustedDocumentsSubmitVariables<'_>,
) -> Result<TrustedDocumentsSubmitPayload, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;
    let operation = TrustedDocumentsSubmit::build(variables);

    let cynic::GraphQlResponse { data, errors } = client.post(&platform_data.api_url).run_graphql(operation).await?;

    if let Some(data) = data {
        Ok(data.trusted_documents_submit)
    } else {
        Err(ApiError::RequestError(format!("{errors:#?}")))
    }
}
