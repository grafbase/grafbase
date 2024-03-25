use std::{collections::HashMap, sync::OnceLock};

use engine::{
    registry::{
        federation::FederationResolver,
        field_set::Selection,
        resolvers::{join::JoinResolver, Resolver},
        type_kinds::SelectionSetTarget,
        MetaField, MetaFieldType, MetaInputValue, MetaType,
    },
    QueryPath, Registry,
};
use engine_parser::types::{BaseType, Type};
use engine_value::argument_set::SerializableArgument;
use indexmap::IndexMap;
use regex::Regex;

use crate::{rules::visitor::RuleError, schema_coord::SchemaCoord};

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

            let Some(fields) = ty.field_map() else {
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
                    &fields.values().collect::<Vec<_>>(),
                    &field.args.values().collect::<Vec<_>>(),
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
    container_fields: &[&MetaField],
    available_arguments: &[&MetaInputValue],
    registry: &Registry,
    coord: SchemaCoord<'_>,
    expected_return_type: &MetaFieldType,
) -> Vec<RuleError> {
    let mut errors = vec![];

    let (destination_field, containing_type) =
        match traverse_join_fields(join, container_fields, available_arguments, registry, coord) {
            Ok(field) => field,
            Err(errors) => return errors,
        };

    // If destination_field is non-null but expected_return is null that's fine...
    if !output_types_are_compatible(&destination_field.ty, expected_return_type, registry) {
        errors.push(RuleError::new(
            vec![],
            format!(
                "{coord} is trying to join with {}.{}, but those fields do not have compatible types",
                containing_type.name(),
                &destination_field.name
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
    container_fields: &[&MetaField],
    available_arguments: &[&MetaInputValue],
    registry: &'a Registry,
    coord: SchemaCoord<'_>,
) -> Result<(&'a MetaField, SelectionSetTarget<'a>), Vec<RuleError>> {
    let mut current_type = registry.root_type(engine_parser::types::OperationType::Query);

    let mut errors = vec![];

    let available_variables = {
        let mut variable_types = HashMap::new();

        // Arguments always shadow container fields so the order of these loops matters
        for field in container_fields {
            variable_types.insert(field.name.as_str(), field.ty.as_str());
        }
        for argument in available_arguments {
            variable_types.insert(argument.name.as_str(), argument.ty.as_str());
        }
        variable_types
    };

    let mut field_iter = join.fields.iter().peekable();
    while let Some((name, join_arguments)) = field_iter.next() {
        let Some(field) = current_type.field(name) else {
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

        for (name, argument) in join_arguments.iter() {
            let Some(arg) = field.args.get(name) else {
                errors.push(RuleError::new(
                    vec![],
                    format!(
                        "{coord} has a join that provides the {} argument to {}.{}, but there is no such argument",
                        name,
                        current_type.name(),
                        &field.name,
                    ),
                ));
                continue;
            };

            validate_join_argument(arg, argument, coord, &available_variables, &mut errors, registry);
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
            Ok(ty) => ty,
            Err(error) => {
                errors.push(RuleError::new(vec![], error.message));
                break;
            }
        };
        match ty.try_into() {
            Ok(ty) => {
                current_type = ty;
            }
            Err(_) => {
                let name = ty.name();
                errors.push(RuleError::new(
                    vec![],
                    format!(
                        "The join on {coord} tries to select children of {name}, but {name} is not a composite type",
                    ),
                ));
                break;
            }
        }
    }

    assert!(!errors.is_empty(), "we shouldnt get here if errors is empty");

    Err(errors)
}

fn validate_join_argument(
    argument_definition: &MetaInputValue,
    argument_value: &SerializableArgument,
    join_coord: SchemaCoord<'_>,
    available_variables: &HashMap<&str, &str>,
    errors: &mut Vec<RuleError>,
    registry: &Registry,
) {
    let argument_type = Type::new(argument_definition.ty.as_str()).expect("valid type strings");

    let mut stack = vec![(argument_type, argument_value, QueryPath::empty())];

    while let Some((current_type, value, path)) = stack.pop() {
        let error_prefix = JoinArgumentErrorPrefix(join_coord, argument_definition.name.as_str(), &path);
        match value {
            SerializableArgument::Variable(variable) => {
                let Some(variable_type) = available_variables.get(variable.as_str()) else {
                    errors.push(RuleError::new(
                        vec![],
                        format!(
                            "{error_prefix} Found the variable {variable} which is not present as a field of the container or an argument of the joined field",
                        ),
                    ));
                    continue;
                };

                if !types_are_compatible(&Type::new(variable_type).expect("valid types"), &current_type, registry) {
                    errors.push(RuleError::new(
                        vec![],
                        format!(
                            "{error_prefix} ${variable} is of type {} but is being used in a position expecting {}",
                            variable_type, current_type
                        ),
                    ));
                    continue;
                }
            }
            SerializableArgument::Null if current_type.nullable => {}
            SerializableArgument::Null => {
                errors.push(RuleError::new(
                    vec![],
                    format!("{error_prefix} Found null where we expected {}", current_type),
                ));
                break;
            }
            SerializableArgument::List(values) if current_type.base.is_list() => {
                let BaseType::List(inner_type) = current_type.base else {
                    unreachable!()
                };

                stack.extend(values.iter().enumerate().map(|(index, value)| {
                    let mut inner_path = path.clone();
                    inner_path.push(index);
                    (*inner_type.clone(), value, inner_path)
                }))
            }
            SerializableArgument::List(_) => {
                errors.push(RuleError::new(
                    vec![],
                    format!("{error_prefix} Found a list where we expected {}", current_type),
                ));
                break;
            }
            SerializableArgument::Object(object) => {
                let BaseType::Named(type_name) = current_type.base else {
                    errors.push(RuleError::new(
                        vec![],
                        format!("{error_prefix}. Found an object where we expected {}", current_type),
                    ));
                    break;
                };

                let Ok(MetaType::InputObject(input_object)) = registry.lookup(&type_name) else {
                    errors.push(RuleError::new(
                        vec![],
                        format!("{error_prefix}.  Found an object where we expected a {}", type_name,),
                    ));
                    break;
                };

                for (field_name, value) in object {
                    let field_name = field_name.as_str();

                    let mut inner_path = path.clone();
                    inner_path.push(field_name);

                    let field_type = match input_object.input_fields.get(field_name) {
                        Some(field) => Type::new(field.ty.as_str()).expect("valid type strings"),
                        None => {
                            errors.push(RuleError::new(
                                vec![],
                                format!("{error_prefix} Could not find a field named {}", field_name),
                            ));
                            continue;
                        }
                    };

                    stack.push((field_type, value, inner_path));
                }
            }
            SerializableArgument::Enum(_) => {
                let BaseType::Named(type_name) = current_type.base else {
                    errors.push(RuleError::new(
                        vec![],
                        format!("{error_prefix} Found an enum where we expected {}", current_type),
                    ));
                    break;
                };

                let Ok(MetaType::Enum(_)) = registry.lookup(&type_name) else {
                    errors.push(RuleError::new(
                        vec![],
                        format!("{error_prefix} Found an enum where we expected a {}", type_name),
                    ));
                    break;
                };
            }
            SerializableArgument::Number(_)
            | SerializableArgument::String(_)
            | SerializableArgument::Boolean(_)
            | SerializableArgument::Binary(_) => {
                let BaseType::Named(type_name) = &current_type.base else {
                    errors.push(RuleError::new(
                        vec![],
                        format!("{error_prefix} Found a scalar where we expected {}", current_type),
                    ));
                    break;
                };

                let Ok(MetaType::Scalar(_)) = registry.lookup(type_name) else {
                    errors.push(RuleError::new(
                        vec![],
                        format!("{error_prefix} Found a scalar where we expected {}", current_type),
                    ));
                    break;
                };
            }
        }
    }
}

fn output_types_are_compatible(
    actual_type: &MetaFieldType,
    expected_type: &MetaFieldType,
    registry: &Registry,
) -> bool {
    let Some(actual) = Type::new(actual_type.as_str()) else {
        return false;
    };
    let Some(expected) = Type::new(expected_type.as_str()) else {
        return false;
    };
    types_are_compatible(&actual, &expected, registry)
}

fn types_are_compatible(mut actual: &Type, mut expected: &Type, registry: &Registry) -> bool {
    loop {
        if actual.nullable && !expected.nullable {
            return false;
        }
        match (&actual.base, &expected.base) {
            (BaseType::List(actual_inner), BaseType::List(expected_inner)) => {
                actual = actual_inner.as_ref();
                expected = expected_inner.as_ref();
            }
            (BaseType::Named(actual_name), BaseType::Named(expected_name)) => {
                if actual_name == expected_name {
                    return true;
                }
                match registry
                    .lookup(actual_name)
                    .ok()
                    .zip(registry.lookup(expected_name).ok())
                {
                    Some((MetaType::Scalar(_), MetaType::Scalar(_))) => {
                        // If the names don't match but both sides are scalars we'll say they're compatible.
                        // This maybe isn't strictly correct but a bit of flexibility around
                        // passsing strings to ID fields and handling mismatched custom scalars
                        // seems sensible.
                        return true;
                    }
                    Some(
                        (MetaType::InputObject(_), MetaType::Object(_))
                        | (MetaType::Object(_), MetaType::InputObject(_)),
                    ) => {
                        // Likewise, it's possible for input objects & objects to be compatible
                        // even though they're different types. We could probably validate this
                        // but its work and I need to get onto other things so _for now_ lets just
                        // hope the user has done the right thing.
                        return true;
                    }
                    _ => return false,
                }
            }
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
        let Some(fields) = ty.fields() else { continue };

        for key in entity.keys() {
            if let Some(FederationResolver::Join(join)) = key.resolver() {
                errors.extend(validate_join(
                    join,
                    &fields
                        .values()
                        .filter(|field| key.includes_field(&field.name))
                        .collect::<Vec<_>>(),
                    &[],
                    registry,
                    SchemaCoord::Entity(ty.name(), &key.to_string()),
                    &ty.name().into(),
                ));
            }
        }
    }

    errors
}

pub struct JoinArgumentErrorPrefix<'a>(SchemaCoord<'a>, &'a str, &'a QueryPath);

impl std::fmt::Display for JoinArgumentErrorPrefix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The join on {} has an invalid value for argument {}", self.0, self.1)?;

        if !self.2.is_empty() {
            write!(f, " (at position {})", self.2)?;
        }

        write!(f, ".")
    }
}
