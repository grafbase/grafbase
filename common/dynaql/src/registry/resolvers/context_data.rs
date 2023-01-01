#![allow(deprecated)]

use dynamodb::{attribute_to_value, ParentRelationId};
use graph_entities::cursor::PaginationCursor;

use super::{ResolvedValue, Resolver};
use crate::dynamic::DynamicFieldContext;

use crate::Error;
use std::hash::Hash;

#[non_exhaustive]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, Hash, PartialEq, Eq)]
pub enum ParentDataResolver {
    Clone,
    ConvertSkToCursor,
    DynamoAttribute(String),
    Field(String),
    PageInfo,
    /// Used for nested pagination to propagate the relation & parent id.
    ParentRelationId {
        relation_name: String,
        parent_id: Resolver,
    },
}

impl ParentDataResolver {
    pub async fn resolve<'ctx>(
        &self,
        ctx_field: &DynamicFieldContext<'ctx>,
        maybe_parent_value: Option<&ResolvedValue<'ctx>>,
    ) -> Result<ResolvedValue<'ctx>, Error> {
        match self {
            Self::ConvertSkToCursor => {
                let value = maybe_parent_value
                    .map(|pv| &pv.value)
                    .ok_or_else(|| Error::new("Internal Error: No parent value exist"))?;

                let sk: String = serde_json::from_value(value.clone().into_owned())?;
                Ok(ResolvedValue::owned(serde_json::to_value(
                    PaginationCursor { sk },
                )?))
            }
            Self::DynamoAttribute(key) => {
                let value = maybe_parent_value
                    .map(|pv| &pv.value)
                    .ok_or_else(|| Error::new("Internal Error: No parent value exist"))?;

                Ok(ResolvedValue::owned(
                    if let Some(attribute) = value.get(key) {
                        attribute_to_value(serde_json::from_value(attribute.clone())?)
                    } else {
                        serde_json::Value::Null
                    },
                ))
            }
            ParentDataResolver::Field(key) => {
                let value = maybe_parent_value
                    .map(|pv| &pv.value)
                    .ok_or_else(|| Error::new("Internal Error: No parent value exist"))?;

                Ok(ResolvedValue::owned(
                    value.get(key).cloned().unwrap_or(serde_json::Value::Null),
                ))
            }
            ParentDataResolver::PageInfo => {
                let pagination = maybe_parent_value
                    .and_then(|pv| pv.pagination.as_ref())
                    .ok_or_else(|| Error::new("Internal Error: No parent pagination exist"))?;

                Ok(ResolvedValue::owned(serde_json::to_value(
                    pagination.output(),
                )?))
            }
            ParentDataResolver::Clone => {
                let parent_value = maybe_parent_value
                    .ok_or_else(|| Error::new("Internal Error: No parent value exist"))?;
                Ok(parent_value.clone())
            }
            ParentDataResolver::ParentRelationId {
                relation_name,
                parent_id,
            } => Ok(ResolvedValue::owned(serde_json::to_value(Some(
                ParentRelationId {
                    relation_name: relation_name.to_string(),
                    parent_id: parent_id
                        .resolve::<String>(ctx_field, maybe_parent_value)
                        .await?,
                },
            ))?)),
        }
    }
}
