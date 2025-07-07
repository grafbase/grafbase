use runtime::trusted_documents_client::TrustedDocumentsEnforcementMode;

use crate::AccessToken;

pub struct TrustedDocumentsClientConfig<'a> {
    pub branch_id: ulid::Ulid,
    pub bypass_header: Option<(String, String)>,
    pub enforcement_mode: TrustedDocumentsEnforcementMode,
    pub object_storage_url: url::Url,
    pub access_token: &'a AccessToken,
}

pub(crate) struct TrustedDocumentsClient {
    /// The base URL for the object storage service.
    object_storage_url: url::Url,

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
    pub(crate) fn new(config: TrustedDocumentsClientConfig<'_>) -> Self {
        let TrustedDocumentsClientConfig {
            branch_id,
            bypass_header,
            enforcement_mode,
            object_storage_url,
            access_token,
        } = config;

        let mut headers = http::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {access_token}")
                .parse()
                .expect("Access token to be a valid header value"),
        );

        let http_client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("Trusted documents HTTP client to initialize");

        Self {
            object_storage_url,
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
        let url = trusted_document_url(&self.object_storage_url, branch_id, client_name, document_id);

        let response = self
            .http_client
            .get(url)
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

fn trusted_document_url(assets_host: &url::Url, branch_id: ulid::Ulid, client_name: &str, document_id: &str) -> String {
    let mut url = assets_host.clone();
    let path = format!("trusted-documents/branch/{branch_id}/{client_name}/{document_id}");
    url.set_path(&path);
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_documents_object_storage_path_basic() {
        let assets_host: url::Url = "http://example.com".parse().unwrap();
        let branch_id = ulid::Ulid::from_parts(1394832, 912849832);
        let client_name = "my-demo-client";
        let document_id = "b1249cf1281";

        assert_eq!(
            trusted_document_url(&assets_host, branch_id, client_name, document_id),
            "http://example.com/trusted-documents/branch/000001AJ4G0000000000V6HYX8/my-demo-client/b1249cf1281"
        )
    }

    #[test]
    fn trusted_documents_object_storage_path_with_space() {
        let assets_host: url::Url = "http://example.com".parse().unwrap();
        let branch_id = ulid::Ulid::from_parts(1394832, 912849832);
        let client_name = "Grafbase Dashboard";
        let document_id = "b1249cf1281ffffffff";

        assert_eq!(
            trusted_document_url(&assets_host, branch_id, client_name, document_id),
            "http://example.com/trusted-documents/branch/000001AJ4G0000000000V6HYX8/Grafbase%20Dashboard/b1249cf1281ffffffff"
        )
    }
}
