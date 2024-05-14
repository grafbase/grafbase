use std::sync::OnceLock;

use engine::{
    registry::{field_set::FieldSetDisplay, resolvers::Resolver, MetaField, MetaFieldType},
    Pos, Registry,
};
use engine_parser::types::{BaseType, Type};
use indexmap::IndexMap;
use regex::Regex;
use registry_v1::ObjectType;
use registry_v2::{resolvers::join::JoinResolver, FederationResolver, Selection};

use crate::{
    rules::{federation::KeyDirective, visitor::RuleError},
    schema_coord::SchemaCoord,
};

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
    errors.extend(validate_federation_joins(registry));

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
                    SchemaCoord::Field(ty.name(), &field.name),
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
    coord: SchemaCoord<'_>,
    current_type: &str,
) -> Vec<RuleError> {
    let mut errors = vec![];
    for selection in required_fields {
        let Some(field) = available_fields.get(&selection.field) else {
            errors.push(RuleError::new(
                vec![],
                format!(
                    "{coord} declares that it requires the field {} on {current_type} but that field doesn't exist",
                    &selection.field
                ),
            ));
            continue;
        };

        if !selection.selections.is_empty() {
            let ty = registry
                .lookup(&field.ty)
                .expect("all the registry types to actually exist");

            let Some(fields) = ty.fields() else {
                errors.push(RuleError::new(
                    vec![],
                    format!(
                        "{coord} tries to require subfields of {} on {current_type} but that field is a leaf type",
                        &selection.field
                    ),
                ));
                continue;
            };

            errors.extend(validate_single_require(
                &selection.selections,
                fields,
                registry,
                coord,
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
        let Some(fields) = ty.fields() else { continue };

        for field in fields.values() {
            if let Resolver::Join(join) = &field.resolver {
                errors.extend(validate_join(
                    join,
                    registry,
                    SchemaCoord::Field(ty.name(), &field.name),
                    &field.ty,
                ));
            }
        }
    }

    errors
}

fn validate_join(
    join: &JoinResolver,
    registry: &Registry,
    coord: SchemaCoord<'_>,
    expected_return_type: &MetaFieldType,
) -> Vec<RuleError> {
    let mut errors = vec![];

    let (destination_field, containing_type) = match traverse_join_fields(join, registry, coord) {
        Ok(field) => field,
        Err(errors) => return errors,
    };

    // If destination_field is non-null but expected_return is null that's fine...
    if !types_are_compatible(&destination_field.ty, expected_return_type) {
        errors.push(RuleError::new(
            vec![],
            format!(
                "{coord} is trying to join with {}.{}, but those fields do not have compatible types: '{}' and '{}'",
                containing_type.name(),
                &destination_field.name,
                destination_field.ty,
                expected_return_type
            ),
        ));
    }

    // I'd like to check that the argument types matches, but unfortunately
    // we don't have type information by the time we get here...

    errors
}

/// Traverses all the fields involved in a join, validating as we go.
///
/// Will return the target MetaField if succesful, errors if not
fn traverse_join_fields<'a>(
    join: &JoinResolver,
    registry: &'a Registry,
    coord: SchemaCoord<'_>,
) -> Result<(&'a MetaField, &'a registry_v1::MetaType), Vec<RuleError>> {
    let mut current_type = registry.root_type(registry_v2::OperationType::Query);

    let mut errors = vec![];

    let mut field_iter = join.fields.iter().peekable();
    while let Some((name, join_arguments)) = field_iter.next() {
        let Some(field) = current_type.field_by_name(name) else {
            errors.push(RuleError::new(
                vec![],
                format!(
                    "{coord} is trying to join with a field named {}, which doesn't exist on the {} type",
                    name,
                    current_type.name()
                ),
            ));
            break;
        };

        for (name, argument) in &field.args {
            if argument.ty.is_non_null() && !join_arguments.contains_argument(name) {
                errors.push(RuleError::new(
                    vec![],
                    format!(
                        "{coord} is trying to join with {}.{}, but does not provide the non-nullable argument {name}",
                        current_type.name(),
                        &field.name,
                    ),
                ));
            }
        }

        for name in join_arguments.iter_names() {
            if !field.args.contains_key(name) {
                errors.push(RuleError::new(
                    vec![],
                    format!(
                        "{coord} has a join that provides the {} argument to {}.{}, but there is no such argument",
                        name,
                        current_type.name(),
                        &field.name,
                    ),
                ));
            }
        }

        if field_iter.peek().is_none() {
            if errors.is_empty() {
                return Ok((field, current_type));
            }
            break;
        }

        if field.ty.is_list() {
            errors.push(RuleError::new(
                vec![],
                format!(
                    "The join on {coord} passes through {}.{}, which is a list.  This is not supported",
                    current_type.name(),
                    &field.name,
                ),
            ));
        }

        // Lookup the type for the next iteration
        let ty = match registry.lookup(&field.ty) {
            Some(ty) => ty,
            _error => {
                errors.push(RuleError::new(
                    vec![],
                    format!("Could not find type named {}", field.ty.base_type_name()),
                ));
                break;
            }
        };
        if !ty.is_composite() {
            let name = ty.name();
            errors.push(RuleError::new(
                vec![],
                format!("The join on {coord} tries to select children of {name}, but {name} is not a composite type",),
            ));
            break;
        }
        current_type = ty;
    }

    assert!(!errors.is_empty(), "we shouldnt ger here if errors is empty");

    Err(errors)
}

fn types_are_compatible(actual_type: &MetaFieldType, expected_type: &MetaFieldType) -> bool {
    let Some(actual) = Type::new(actual_type.as_str()) else {
        return false;
    };
    let Some(expected) = Type::new(expected_type.as_str()) else {
        return false;
    };

    let mut actual = &actual;
    let mut expected = &expected;

    loop {
        if actual.nullable && !expected.nullable {
            return false;
        }
        match (&actual.base, &expected.base) {
            (BaseType::List(actual_inner), BaseType::List(expected_inner)) => {
                actual = actual_inner.as_ref();
                expected = expected_inner.as_ref();
            }
            (BaseType::Named(actual_name), BaseType::Named(expected_name)) => return actual_name == expected_name,
            _ => {
                // I've not implemented the list coercion rules here but since
                // this is just used for joins I think we're fine without them for now.
                // Can add later if that turns out to be wrong
                return false;
            }
        }
    }
}

/// Validates that all the federation joins in the schema make sense
fn validate_federation_joins(registry: &Registry) -> Vec<RuleError> {
    let mut errors = vec![];

    for (name, entity) in &registry.federation_entities {
        let Some(ty) = registry.types.get(name) else { continue };

        for key in entity.keys.iter() {
            if let Some(FederationResolver::Join(join)) = key.resolver() {
                errors.extend(validate_join(
                    join.as_ref(),
                    registry,
                    SchemaCoord::Entity(ty.name(), &FieldSetDisplay(&key.selections).to_string()),
                    &ty.name().into(),
                ));
            }
        }
    }

    errors
}

pub fn validate_federation_keys(pos: Pos, key: &KeyDirective, object: &ObjectType) -> Vec<RuleError> {
    let mut errors = Vec::new();

    errors.extend(key.validate().into_iter().map(|error| RuleError::new(vec![pos], error)));

    for field in &key.fields.0 .0 {
        if object.field_by_name(&field.field).is_none() {
            errors.push(RuleError::new(
                vec![pos],
                format!(
                    "The object {} has a key that requires the field {} but that field isn't present",
                    object.name, &field.field,
                ),
            ));
        }
        // In an ideal world we'd also validate any nested keys, but I don't want
        // to write that code today.  If you would like to fix this, feel free.
    }

    errors
}
