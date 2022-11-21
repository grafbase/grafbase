//! Transformers are a part of the Resolving Logic, Transformers are applied in
//! the resolving, after executing the async code to resolve a field, we apply
//! the transformers associated.
//!
//! Each Transformer function is defined to do:
//! serde_json::Value -> serde_json::Value
//!
//! The transform step is synchronous.
//!
//! At the end of the transformation, each transformed values are merged into one
//! serde_json::Value.

use super::utils::attribute_to_value;
use crate::Error;
use dynomite::AttributeValue;
use graph_entities::cursor::PaginationCursor;
use std::collections::HashMap;

/// Describe the Transformer step used to transform a Value from the Resolver.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum Transformer {
    ConvertPkToCursor,
    DynamoSelect {
        /// The key where this select
        property: String,
    },
    JSONSelect {
        /// The key where this select
        property: String,
    },
    Pipeline(Vec<Transformer>),
}

impl Transformer {
    pub fn transform(&self, value: serde_json::Value) -> Result<serde_json::Value, Error> {
        match &self {
            Self::Pipeline(transformers) => {
                transformers.iter().try_fold(value, |v, t| t.transform(v))
            }
            Self::ConvertPkToCursor => {
                let pk: String = serde_json::from_value(value)?;
                Ok(serde_json::to_value(PaginationCursor { pk })?)
            }
            Self::JSONSelect { property } => {
                let result = value
                    .get(property)
                    .map(std::borrow::ToOwned::to_owned)
                    .unwrap_or_else(|| serde_json::Value::Null);
                Ok(result)
            }
            Self::DynamoSelect { property } => {
                let cast: Option<HashMap<String, AttributeValue>> = serde_json::from_value(value)?;

                let result = cast
                    .map(|mut x| x.remove(property))
                    .flatten()
                    .map(attribute_to_value)
                    .unwrap_or_else(|| serde_json::Value::Null);

                Ok(result)
            }
        }
    }
}
