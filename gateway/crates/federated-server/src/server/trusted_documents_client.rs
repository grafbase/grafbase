const GRAFBASE_PRODUCTION_TRUSTED_DOCUMENTS_BUCKET: &str = "https://pub-72f3517515a34104921bb714721a885a.r2.dev";

pub(crate) struct TrustedDocumentsClient {
    pub(crate) http_client: reqwest::Client,
    pub(crate) branch_id: ulid::Ulid,
    pub(crate) bypass_header: Option<(String, String)>,
}

#[async_trait::async_trait]
impl runtime::trusted_documents_client::TrustedDocumentsClient for TrustedDocumentsClient {
    fn is_enabled(&self) -> bool {
        true
    }

    fn bypass_header(&self) -> Option<(&str, &str)> {
        self.bypass_header
            .as_ref()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    async fn fetch(
        &self,
        client_name: &str,
        document_id: &str,
    ) -> runtime::trusted_documents_client::TrustedDocumentsResult<String> {
        let branch_id = self.branch_id;
        let key = format!("{branch_id}/{client_name}/{document_id}");

        let mut url: url::Url = GRAFBASE_PRODUCTION_TRUSTED_DOCUMENTS_BUCKET.parse().unwrap();
        url.set_path(&key);

        let response = self
            .http_client
            .get(&url.to_string())
            .send()
            .await
            .map_err(|err| runtime::trusted_documents_client::TrustedDocumentsError::RetrievalError(err.into()))?;

        if !response.status().is_success() {
            return Err(runtime::trusted_documents_client::TrustedDocumentsError::DocumentNotFound);
        }

        let document = response
            .text()
            .await
            .map_err(|err| runtime::trusted_documents_client::TrustedDocumentsError::RetrievalError(err.into()))?;

        Ok(document)
    }
}
