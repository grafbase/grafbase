//! # Grafbase Operation Normalizer
//!
//! A library to modify a incoming query so we can match similar queries to be the
//! same, even if they have differences in their string representation. This is achieved
//! by doing the following steps:
//!
//! - Removal of all hard-coded arguments for the following types:
//!   - String (replace with "")
//!   - Float (replace with 0.0)
//!   - Int (replace with 0)
//!   - List (replace with [])
//!   - Object (replace with {})
//! - Leave parameters, enums and booleans as-is
//! - Remove all fragments not used in the query
//! - Remove all comments
//! - Reorder fields, arguments, selections in alphabetic order
//! - Parse and render, removing extra whitespace and other stylistic things

#![deny(missing_docs)]

use grafbase_workspace_hack as _;

mod arguments;
mod directives;
mod operation;
mod selection_set;

#[cfg(test)]
mod tests;

use std::{cmp::Ordering, collections::HashMap};

use graphql_parser::query::{Definition, OperationDefinition};

/// With the given input, returns a normalized output following the operation signature rules.
///
/// - If the incoming operation is named, the source must have an operation with a given name.
/// - For unnamed operations, the source must include only a single operation and that cannot be named.
/// - The schema must parse and validate as an executable query document.
pub fn normalize(source_text: &str, operation_name: Option<&str>) -> anyhow::Result<String> {
    let mut document = graphql_parser::parse_query::<&str>(source_text)?;
    let mut used_fragments = HashMap::new();

    if let Some(operation_name) = operation_name {
        document.definitions.retain(|definition| match definition {
            Definition::Operation(OperationDefinition::Query(query)) => query.name == Some(operation_name),
            Definition::Operation(OperationDefinition::Mutation(mutation)) => mutation.name == Some(operation_name),
            Definition::Operation(OperationDefinition::Subscription(subscription)) => {
                subscription.name == Some(operation_name)
            }
            _ => true,
        });
    }

    // iterate over operations first, so we know what fragments are in use
    for definition in &mut document.definitions {
        if let Definition::Operation(operation) = definition {
            operation::normalize(operation, &mut used_fragments)?;
        }
    }

    // and now we can normalize and map fragments which we know are used
    // in operations
    for definition in &mut document.definitions {
        if let Definition::Fragment(fragment) = definition {
            let in_operation = used_fragments.contains_key(fragment.name);

            if !in_operation {
                continue;
            }

            directives::normalize(&mut fragment.directives);
            selection_set::normalize(&mut fragment.selection_set, &mut used_fragments, in_operation);
        }
    }

    document.definitions.retain(|definition| match definition {
        Definition::Fragment(fragment) => *used_fragments.get(fragment.name).unwrap_or(&false),
        _ => true,
    });

    document.definitions.sort_by(|a, b| {
        match (a, b) {
            (Definition::Operation(_), Definition::Fragment(_)) => Ordering::Greater,
            (Definition::Fragment(_), Definition::Operation(_)) => Ordering::Less,
            (Definition::Fragment(a), Definition::Fragment(b)) => a.name.cmp(b.name),
            // here we only have one operation left, all the others are normalized out
            (Definition::Operation(_), Definition::Operation(_)) => Ordering::Equal,
        }
    });

    if document.definitions.is_empty() {
        anyhow::bail!("the normalized query is empty (meaning we couldn't find an operation with the given name)");
    } else {
        Ok(document.to_string())
    }
}
