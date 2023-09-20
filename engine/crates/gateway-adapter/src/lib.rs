use std::collections::HashMap;

pub use self::{
    execution_engine::{ExecutionEngine, ExecutionError, ExecutionResult},
    registry::{RegistryError, RegistryProvider, RegistryResult},
};
pub use engine::registry::VersionedRegistry;

use common_types::auth::ExecutionAuth;

mod execution_engine;
mod registry;

/// Owned execution request
#[derive(serde::Deserialize, serde::Serialize)]
pub struct ExecutionRequest {
    /// The request to execute
    pub request: engine::Request,
    /// Authorization details
    pub auth: ExecutionAuth,

    #[serde(skip)]
    /// AWS Region closest to the worker Colocation
    pub closest_aws_region: rusoto_core::Region,
    /// Request headers
    pub execution_headers: HashMap<String, String>,
}
