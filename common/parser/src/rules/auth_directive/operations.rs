use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::rust::sets_duplicate_value_is_error;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Operations(#[serde(with = "sets_duplicate_value_is_error")] HashSet<Operation>);

impl std::iter::FromIterator<Operation> for Operations {
    fn from_iter<I: IntoIterator<Item = Operation>>(iter: I) -> Self {
        Operations(iter.into_iter().collect())
    }
}

impl Default for Operations {
    fn default() -> Self {
        [Operation::Create, Operation::Read, Operation::Update, Operation::Delete]
            .into_iter()
            .collect()
    }
}

impl Operations {
    pub fn values(&self) -> &HashSet<Operation> {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum Operation {
    Create,
    Read,
    Get,  // More granual read access
    List, // More granual read access
    Update,
    Delete,
}

impl From<Operations> for grafbase::auth::Operations {
    fn from(ops: Operations) -> Self {
        let mut res = Self::empty();
        for op in ops.0 {
            res |= match op {
                Operation::Create => Self::CREATE,
                Operation::Read => Self::READ,
                Operation::Get => Self::GET,
                Operation::List => Self::LIST,
                Operation::Update => Self::UPDATE,
                Operation::Delete => Self::DELETE,
            };
        }
        res
    }
}
