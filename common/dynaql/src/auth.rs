use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::rust::sets_duplicate_value_is_error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auth {
    pub allow_anonymous_access: bool,
    pub allowed_anonymous_ops: Operations,

    pub allow_private_access: bool,
    pub allowed_private_ops: Operations,

    #[serde(with = "sets_duplicate_value_is_error")]
    pub allowed_groups: HashSet<String>,
    pub allowed_group_ops: Operations,

    pub oidc_providers: Vec<OidcProvider>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidcProvider {
    pub issuer: url::Url,
}

impl Default for Auth {
    fn default() -> Self {
        Auth {
            allow_anonymous_access: true,
            allowed_anonymous_ops: Operations::all(),

            allow_private_access: false,
            allowed_private_ops: Operations::none(),

            allowed_groups: HashSet::new(),
            allowed_group_ops: Operations::none(),

            oidc_providers: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct Operations(#[serde(with = "sets_duplicate_value_is_error")] HashSet<Operation>);

impl std::iter::FromIterator<Operation> for Operations {
    fn from_iter<I: IntoIterator<Item = Operation>>(iter: I) -> Self {
        Operations(iter.into_iter().collect())
    }
}

impl Operations {
    pub fn all() -> Self {
        Operations(
            vec![
                Operation::Create,
                Operation::Read,
                Operation::Update,
                Operation::Delete,
            ]
            .into_iter()
            .collect(),
        )
    }

    pub fn none() -> Self {
        Operations::default()
    }

    pub fn values(&self) -> &HashSet<Operation> {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Create,
    Read,
    Get,  // More granual read access
    List, // More granual read access
    Update,
    Delete,
}
