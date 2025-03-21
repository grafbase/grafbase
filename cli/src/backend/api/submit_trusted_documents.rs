pub(crate) use crate::backend::api::graphql::mutations::submit_trusted_documents::{
    ReusedIds, TrustedDocumentInput, TrustedDocumentsSubmitPayload, TrustedDocumentsSubmitVariables,
};

use crate::{
    backend::api::{
        client::create_client, errors::ApiError, graphql::mutations::submit_trusted_documents::TrustedDocumentsSubmit,
    },
    common::environment::PlatformData,
};
use cynic::{MutationBuilder, http::ReqwestExt};

#[tokio::main]
pub async fn submit_trusted_documents(
    variables: TrustedDocumentsSubmitVariables<'_>,
) -> Result<TrustedDocumentsSubmitPayload, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;
    let operation = TrustedDocumentsSubmit::build(variables);

    let cynic::GraphQlResponse { data, errors } = client.post(platform_data.api_url()).run_graphql(operation).await?;

    if let Some(data) = data {
        Ok(data.trusted_documents_submit)
    } else {
        Err(ApiError::RequestError(format!("{errors:#?}")))
    }
}
