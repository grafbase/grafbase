use std::sync::OnceLock;

use engine::{
    registry::{
        field_set::Selection,
        resolvers::{join::JoinResolver, Resolver},
        MetaField,
    },
    Registry,
};
use indexmap::IndexMap;
use regex::Regex;

use crate::rules::visitor::RuleError;

static NAME_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn validate_connector_name(name: &str) -> Result<(), String> {
    let name_regex = NAME_REGEX.get_or_init(|| Regex::new("^[A-Za-z_][A-Za-z0-9_]*$").unwrap());

    if name.is_empty() {
        return Err("Connector names cannot be empty".into());
    }

    if !name_regex.is_match(name) {
        return Err("Connector names must be alphanumeric and cannot start with a number".into());
    }

    Ok(())
}

pub fn post_parsing_validations(registry: &Registry) -> Vec<RuleError> {
    let mut errors = vec![];

    errors.extend(validate_all_requires(registry));
    errors.extend(validate_joins(registry));

    errors
}

/// Validate that the fields requested in all the @requires in the registry actually exist.
fn validate_all_requires(registry: &Registry) -> Vec<RuleError> {
    let mut errors = vec![];

    for ty in registry.types.values() {
        let Some(fields) = ty.fields() else {
            continue;
        };
        for field in fields.values() {
            if let Some(field_set) = &field.requires {
                errors.extend(validate_single_require(
                    &field_set.0,
                    fields,
                    registry,
                    ty.name(),
                    &field.name,
                    ty.name(),
                ));
            }
        }
    }

    errors
}

fn validate_single_require(
    required_fields: &[Selection],
    available_fields: &IndexMap<String, MetaField>,
    registry: &Registry,
    type_with_requires: &str,
    field_with_requires: &str,
    current_type: &str,
) -> Vec<RuleError> {
    let mut errors = vec![];
    for selection in required_fields {
        let Some(field) = available_fields.get(&selection.field) else {
            errors.push(RuleError::new(vec![], format!("The field {field_with_requires} on {type_with_requires} declares that it requires the field {} on {current_type} but that field doesn't exist", &selection.field)));
            continue;
        };

        if !selection.selections.is_empty() {
            let ty = registry
                .lookup(&field.ty)
                .expect("all the registry types to actually exist");

            let Some(fields) = ty.field_map() else {
                errors.push(RuleError::new(vec![], format!("The field {field_with_requires} on {type_with_requires} tries to require subfields of {} on {current_type} but that field is a leaf type", &selection.field)));
                continue;
            };

            errors.extend(validate_single_require(
                &selection.selections,
                fields,
                registry,
                type_with_requires,
                field_with_requires,
                ty.name(),
            ));
        }
    }

    errors
}

/// Validates that all the joins in the schema make sense
fn validate_joins(registry: &Registry) -> Vec<RuleError> {
    let mut errors = vec![];

    for ty in registry.types.values() {
        let Some(fields) = ty.fields() else {
            continue;
        };
        for field in fields.values() {
            if let Resolver::Join(join) = &field.resolver {
                errors.extend(validate_join(join, registry, ty.name(), field));
            }
        }
    }

    errors
}

fn validate_join(join: &JoinResolver, registry: &Registry, type_with_join: &str, field: &MetaField) -> Vec<RuleError> {
    let field_with_join = &field.name;
    let mut errors = vec![];

    let root_query_type = registry.root_type(engine_parser::types::OperationType::Query);
    let Some(destination_field) = root_query_type.field(&join.field_name) else {
        errors.push(RuleError::new(
            vec![],
            format!("The field {field_with_join} of the type {type_with_join} is trying to join with a field named {}, which doesn't exist on the {} type", join.field_name, root_query_type.name()),
        ));
        return errors;
    };

    if field.ty != destination_field.ty {
        errors.push(RuleError::new(
            vec![],
        format!("The field {field_with_join} of the type {type_with_join} is trying to join with the field named {}, but those fields do not have the same type", join.field_name),
        ));
    }

    for (name, argument) in &destination_field.args {
        if argument.ty.is_non_null() && !join.arguments.contains_argument(name) {
            errors.push(RuleError::new(
                vec![],
            format!("The field {field_with_join} of the type {type_with_join} is trying to join with the field named {}, but does not provide the non-nullable argument {name}", join.field_name),
            ));
        }

        // I'd like to check that the argument type matches, but unfortunately
        // we don't have type information by the time we get here...
    }

    errors
}
