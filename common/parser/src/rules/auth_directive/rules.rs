use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use dynaql::ServerError;
use dynaql_value::ConstValue;

use super::operations::Operations;
use crate::VisitorContext;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "allow")]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub enum AuthRule {
    /// Public data access via API keys
    // Ex: { allow: anonymous }
    #[serde(alias = "public")]
    #[serde(rename_all = "camelCase")]
    Anonymous {
        // Note: we don't support operations as our playground needs full access
    },

    /// Signed-in user data access via OIDC
    // Ex: { allow: private }
    //     { allow: private, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Private {
        #[serde(default)]
        operations: Operations,
    },

    /// User group-based data access via OIDC
    // Ex: { allow: groups, groups: ["admin"] }
    //     { allow: groups, groups: ["admin"], operations: [update, delete] }
    #[serde(rename_all = "camelCase")]
    Groups {
        #[serde(with = "::serde_with::rust::sets_duplicate_value_is_error")]
        groups: HashSet<String>,

        #[serde(default)]
        operations: Operations,
    },

    /// Owner-based data access via OIDC
    // Ex: { allow: owner }
    //     { allow: owner, operations: [create, read] }
    #[serde(rename_all = "camelCase")]
    Owner {
        #[serde(default)]
        operations: Operations,
    },
}

impl AuthRule {
    pub fn from_value(_ctx: &VisitorContext<'_>, value: &ConstValue) -> Result<Self, ServerError> {
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

        Ok(rule)
    }
}
