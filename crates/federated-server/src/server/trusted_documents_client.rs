use runtime::trusted_documents_client::TrustedDocumentsEnforcementMode;

use super::graph_updater::{DEFAULT_OBJECT_STORAGE_HOST, OBJECT_STORAGE_HOST_ENV_VAR};

pub(crate) struct TrustedDocumentsClient {
    /// The base URL for the assets host.
    assets_host: url::Url,

    /// The HTTP client used for making requests.
    http_client: reqwest::Client,

    /// The unique identifier for the branch associated with the client.
    branch_id: ulid::Ulid,

    /// Optional header for bypassing into trusted document storage.
    bypass_header: Option<(String, String)>,

    enforcement_mode: TrustedDocumentsEnforcementMode,
}

impl TrustedDocumentsClient {
    /// Creates a new instance of `TrustedDocumentsClient`.
    ///
    /// # Arguments
    ///
    /// - `http_client`: The HTTP client used for making requests.
    /// - `branch_id`: The unique identifier for the branch associated with the client.
    /// - `bypass_header`: Optional header. When provided, requests containing the header with the corresponding value can execute arbitrary queries.
    ///
    /// # Returns
    ///
    /// A new instance of `TrustedDocumentsClient`.
    pub(crate) fn new(
        http_client: reqwest::Client,
        branch_id: ulid::Ulid,
        bypass_header: Option<(String, String)>,
        enforcement_mode: TrustedDocumentsEnforcementMode,
    ) -> Self {
        let assets_host: url::Url = std::env::var(OBJECT_STORAGE_HOST_ENV_VAR)
            .unwrap_or_else(|_| DEFAULT_OBJECT_STORAGE_HOST.to_owned())
            .parse()
            .expect("assets url should be valid");

        Self {
            assets_host,
            http_client,
            branch_id,
            bypass_header,
            enforcement_mode,
        }
    }
}

#[async_trait::async_trait]
impl runtime::trusted_documents_client::TrustedDocumentsClient for TrustedDocumentsClient {
    fn enforcement_mode(&self) -> TrustedDocumentsEnforcementMode {
        self.enforcement_mode
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
        let key = format!("trusted-documents/branch/{branch_id}/{client_name}/{document_id}");

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
