const GRAFBASE_PRODUCTION_TRUSTED_DOCUMENTS_BUCKET: &str = "https://pub-72f3517515a34104921bb714721a885a.r2.dev";
const GRAFBASE_ASSETS_URL_ENV_VAR: &str = "GRAFBASE_ASSETS_URL";

pub(crate) struct TrustedDocumentsClient {
    /// The base URL for the assets host.
    assets_host: url::Url,

    /// The HTTP client used for making requests.
    http_client: reqwest::Client,

    /// The unique identifier for the branch associated with the client.
    branch_id: ulid::Ulid,

    /// Optional header for bypassing into trusted document storage.
    bypass_header: Option<(String, String)>,
}

impl TrustedDocumentsClient {
    /// Creates a new instance of `TrustedDocumentsClient`.
    ///
    /// # Arguments
    ///
    /// - `http_client`: The HTTP client used for making requests.
    /// - `branch_id`: The unique identifier for the branch associated with the client.
    /// - `bypass_header`: Optional header for bypassing into trusted document storage.
    ///
    /// # Returns
    ///
    /// A new instance of `TrustedDocumentsClient`.
    pub(crate) fn new(
        http_client: reqwest::Client,
        branch_id: ulid::Ulid,
        bypass_header: Option<(String, String)>,
    ) -> Self {
        let assets_host: url::Url = std::env::var(GRAFBASE_ASSETS_URL_ENV_VAR)
            .unwrap_or(GRAFBASE_PRODUCTION_TRUSTED_DOCUMENTS_BUCKET.to_string())
            .parse()
            .expect("assets url should be valid");

        Self {
            assets_host,
            http_client,
            branch_id,
            bypass_header,
        }
    }
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

        let mut url = self.assets_host.clone();
        url.set_path(&key);

        let response = self
            .http_client
            .get(url.to_string())
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
