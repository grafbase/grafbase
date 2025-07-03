use std::path::PathBuf;

use gateway_config::Config;
use runtime::trusted_documents_client::{Client, TrustedDocumentsEnforcementMode};
use ulid::Ulid;

use crate::{
    AccessToken, ObjectStorageResponse,
    server::trusted_documents_client::{TrustedDocumentsClient, TrustedDocumentsClientConfig},
};

pub struct Graph {
    pub federated_sdl: String,
    pub version_id: Option<Ulid>,
    pub trusted_documents: Option<Client>,
    pub current_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub enum GraphDefinition {
    /// Response from object storage.
    ObjectStorage {
        response: ObjectStorageResponse,
        object_storage_base_url: url::Url,
    },
    /// Response from static file.
    Sdl(Option<PathBuf>, String),
}

impl GraphDefinition {
    /// Converts a `GraphDefinition` into a `Graph`.
    ///
    /// This method processes the graph definition based on its type:
    ///
    /// - For `ObjectStorage` variants, it returns a graph with potential trusted documents support
    /// - For `Sdl` variants, it returns a graph without trusted documents
    ///
    /// Trusted documents will only be enabled for `ObjectStorage` variants when both an access
    /// token is provided and trusted documents are enabled in the configuration.
    pub fn into_graph(self, config: &Config, access_token: Option<&AccessToken>) -> Graph {
        let (response, object_storage_base_url) = match self {
            GraphDefinition::ObjectStorage {
                response,
                object_storage_base_url,
            } => (response, object_storage_base_url),
            GraphDefinition::Sdl(current_dir, federated_sdl) => {
                return Graph {
                    federated_sdl,
                    version_id: None,
                    trusted_documents: None,
                    current_dir,
                };
            }
        };

        let access_token = match access_token {
            Some(access_token) if config.trusted_documents.enabled => access_token,
            _ => {
                return Graph {
                    federated_sdl: response.sdl,
                    version_id: Some(response.version_id),
                    trusted_documents: None,
                    current_dir: None,
                };
            }
        };

        let enforcement_mode = if config.trusted_documents.enforced {
            TrustedDocumentsEnforcementMode::Enforce
        } else {
            TrustedDocumentsEnforcementMode::Allow
        };

        let bypass_header = config
            .trusted_documents
            .bypass_header
            .bypass_header_name
            .as_ref()
            .zip(config.trusted_documents.bypass_header.bypass_header_value.as_ref())
            .map(|(name, value)| (name.clone().into(), String::from(value.as_str())));

        let trusted_documents_client = TrustedDocumentsClient::new(TrustedDocumentsClientConfig {
            branch_id: response.branch_id,
            bypass_header,
            enforcement_mode,
            object_storage_url: object_storage_base_url,
            access_token,
        });

        Graph {
            federated_sdl: response.sdl,
            version_id: Some(response.version_id),
            trusted_documents: Some(Client::new(trusted_documents_client)),
            current_dir: None,
        }
    }
}
