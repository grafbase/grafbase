use std::collections::BTreeSet;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::rust::sets_duplicate_value_is_error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auth {
    pub allowed_anonymous_ops: Operations,

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
            allowed_anonymous_ops: Operations::all(),

            allowed_private_ops: Operations::none(),

            allowed_groups: HashSet::new(),
            allowed_group_ops: Operations::none(),

            oidc_providers: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Operations(#[serde(with = "sets_duplicate_value_is_error")] BTreeSet<Operation>);

impl std::iter::FromIterator<Operation> for Operations {
    fn from_iter<I: IntoIterator<Item = Operation>>(iter: I) -> Self {
        Operations(iter.into_iter().collect())
    }
}

impl Operations {
    pub fn new(ops: &[Operation]) -> Self {
        Operations(ops.iter().copied().collect())
    }

    pub fn values(&self) -> &BTreeSet<Operation> {
        &self.0
    }

    pub fn any(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn contains(&self, op: &Operation) -> bool {
        match op {
            Operation::Get | Operation::List => {
                self.0.contains(op) || self.0.contains(&Operation::Read)
            }
            _ => self.0.contains(op),
        }
    }

    pub fn all() -> Self {
        Self::new(&[
            Operation::Create,
            Operation::Read,
            Operation::Update,
            Operation::Delete,
        ])
    }

    pub fn none() -> Self {
        Operations(BTreeSet::new())
    }
}

#[derive(
    Debug, strum::Display, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy, Clone,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Operation {
    Create,
    Read,
    Get,  // More granual read access
    List, // More granual read access
    Update,
    Delete,
}
