use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use dynaql::ServerError;
use dynaql_value::ConstValue;

use super::operations::Operations;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "allow")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum AuthRule {
    /// Signed-in user data access with a valid JWT token.
    // Ex: { allow: private }
    //     { allow: private, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Private {
        #[serde(default)]
        operations: Option<Operations>,
    },

    /// Public data access
    // Ex: { allow: public }
    //     { allow: public, operations: [read] }
    #[serde(rename_all = "camelCase")]
    Public {
        #[serde(default)]
        operations: Option<Operations>,
    },

    /// Group-based data access. Access is allowed when a group is found in the JWT token.
    // Ex: { allow: groups, groups: ["admin"] }
    //     { allow: groups, groups: ["admin"], operations: [update, delete] }
    #[serde(rename_all = "camelCase")]
    Groups {
        #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
        groups: HashSet<String>,

        #[serde(default)]
        operations: Option<Operations>,
    },

    /// Owner-based data access - document(row) based security. Owner can only see their own documents.
    // Ex: { allow: owner }
    //     { allow: owner, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Owner {
        #[serde(default)]
        operations: Option<Operations>,
    },
}

impl AuthRule {
    pub fn from_value(value: &ConstValue, is_global: bool) -> Result<Self, ServerError> {
        // We convert the value to JSON to leverage serde for deserialization
        let value = match value {
            ConstValue::Object(_) => value
                .clone()
                .into_json()
                .map_err(|err| ServerError::new(err.to_string(), None))?,
            _ => return Err(ServerError::new("auth rule must be an object", None)),
        };

        let rule: AuthRule =
            serde_json::from_value(value).map_err(|err| ServerError::new(format!("auth rule: {err}"), None))?;

        if !is_global && rule.maybe_operations().map(|ops| ops.contains(super::operations::Operation::Introspection)).unwrap_or_default() {
            Err(ServerError::new(
                "introspection rule can be only configured globally",
                None,
            ))
        } else {
            Ok(rule)
        }
    }

    fn maybe_operations(&self) -> Option<&Operations> {
        match self {
            AuthRule::Private { operations }
            | AuthRule::Public { operations }
            | AuthRule::Groups { groups: _, operations }
            | AuthRule::Owner { operations } => operations,
        }
        .as_ref()
    }
}
