use std::collections::HashSet;

use schema::Schema;

use crate::operation::{OperationWalker, SelectionSetWalker};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum OperationLimitExceededError {
    #[error("Query is too complex.")]
    QueryTooComplex,
    #[error("Query is nested too deep.")]
    QueryTooDeep,
    #[error("Query is too high.")]
    QueryTooHigh,
    #[error("Query contains too many root fields.")]
    QueryContainsTooManyRootFields,
    #[error("Query contains too many aliases.")]
    QueryContainsTooManyAliases,
}

pub(super) fn enforce_operation_limits(
    schema: &Schema,
    operation: OperationWalker<'_>,
    request: &engine::Request,
) -> Result<(), OperationLimitExceededError> {
    if request.operation_limits_disabled() {
        return Ok(());
    }

    let selection_set = operation.selection_set();

    if let Some(depth_limit) = schema.operation_limits.depth {
        let max_depth = selection_set.max_depth();
        if max_depth > depth_limit {
            return Err(OperationLimitExceededError::QueryTooDeep);
        }
    }

    if let Some(max_alias_count) = schema.operation_limits.aliases {
        let alias_count = selection_set.alias_count();
        if alias_count > max_alias_count {
            return Err(OperationLimitExceededError::QueryContainsTooManyAliases);
        }
    }

    if let Some(max_root_field_count) = schema.operation_limits.root_fields {
        let root_field_count = selection_set.root_field_count();
        if root_field_count > max_root_field_count {
            return Err(OperationLimitExceededError::QueryContainsTooManyRootFields);
        }
    }

    if let Some(max_height) = schema.operation_limits.height {
        let height = selection_set.height(&mut Default::default());
        if height > max_height {
            return Err(OperationLimitExceededError::QueryTooHigh);
        }
    }

    if let Some(max_complexity) = schema.operation_limits.complexity {
        let complexity = selection_set.complexity();
        if complexity > max_complexity {
            return Err(OperationLimitExceededError::QueryTooComplex);
        }
    }

    Ok(())
}

impl SelectionSetWalker<'_> {
    fn max_depth(&self) -> u16 {
        self.fields()
            .map(|field| {
                field
                    .selection_set()
                    .map(|selection_set| selection_set.max_depth())
                    .unwrap_or_default()
                    + 1
            })
            .max()
            .expect("must be defined")
    }

    fn alias_count(&self) -> u16 {
        self.fields()
            .map(|field| {
                field.alias().is_some() as u16
                    + field
                        .selection_set()
                        .map(|selection_set| selection_set.alias_count())
                        .unwrap_or_default()
            })
            .sum()
    }

    fn root_field_count(&self) -> u16 {
        self.fields().count() as u16
    }

    fn complexity(&self) -> u16 {
        self.fields()
            .map(|field| {
                field
                    .selection_set()
                    .map(|selection_set| selection_set.complexity())
                    .unwrap_or_default()
                    + 1
            })
            .sum()
    }

    // `None` stored in the set means `__typename`.
    fn height(&self, fields_seen: &mut HashSet<Option<schema::FieldDefinitionId>>) -> u16 {
        self.fields()
            .map(|field| {
                (if fields_seen.insert(field.as_ref().definition_id()) {
                    1
                } else {
                    0
                }) + field
                    .selection_set()
                    .map(|selection_set| selection_set.height(&mut HashSet::new()))
                    .unwrap_or_default()
            })
            .sum()
    }
}
