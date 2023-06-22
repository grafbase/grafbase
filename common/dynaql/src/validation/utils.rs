use std::collections::HashSet;

use dynaql_value::{ConstValue, Value};

use crate::context::QueryPathNode;
use crate::registry::scalars::{DynamicScalar, PossibleScalar};
use crate::{registry, QueryPathSegment};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Scope<'a> {
    Operation(Option<&'a str>),
    Fragment(&'a str),
}

fn valid_error(path_node: &QueryPathNode, msg: String) -> String {
    format!("\"{path_node}\", {msg}")
}

pub fn referenced_variables(value: &Value) -> Vec<&str> {
    let mut vars = Vec::new();
    referenced_variables_to_vec(value, &mut vars);
    vars
}

fn referenced_variables_to_vec<'a>(value: &'a Value, vars: &mut Vec<&'a str>) {
    match value {
        Value::Variable(name) => {
            vars.push(name);
        }
        Value::List(values) => values
            .iter()
            .for_each(|value| referenced_variables_to_vec(value, vars)),
        Value::Object(obj) => obj
            .values()
            .for_each(|value| referenced_variables_to_vec(value, vars)),
        _ => {}
    }
}

pub fn is_valid_input_value(
    registry: &registry::Registry,
    type_name: &str,
    value: &ConstValue,
    path_node: QueryPathNode,
) -> Option<String> {
    match registry::MetaTypeName::create(type_name) {
        registry::MetaTypeName::NonNull(type_name) => match value {
            ConstValue::Null => Some(valid_error(
                &path_node,
                format!("expected type \"{type_name}\""),
            )),
            _ => is_valid_input_value(registry, type_name, value, path_node),
        },
        registry::MetaTypeName::List(type_name) => match value {
            ConstValue::List(elems) => elems.iter().enumerate().find_map(|(idx, elem)| {
                is_valid_input_value(
                    registry,
                    type_name,
                    elem,
                    QueryPathNode {
                        parent: Some(&path_node),
                        segment: QueryPathSegment::Index(idx),
                    },
                )
            }),
            ConstValue::Null => None,
            _ => is_valid_input_value(registry, type_name, value, path_node),
        },
        registry::MetaTypeName::Named(type_name) => {
            if let ConstValue::Null = value {
                return None;
            }

            match registry
                .types
                .get(type_name)
                .unwrap_or_else(|| panic!("{type_name} unknown"))
            {
                registry::MetaType::Scalar { .. } => {
                    if let true = PossibleScalar::is_valid(&type_name, &value) {
                        None
                    } else {
                        Some(valid_error(
                            &path_node,
                            format!("expected type \"{type_name}\""),
                        ))
                    }
                }
                registry::MetaType::Enum(registry::EnumType {
                    enum_values,
                    name: enum_name,
                    ..
                }) => match value {
                    ConstValue::Enum(name) => {
                        if !enum_values.contains_key(name.as_str()) {
                            Some(valid_error(
                                &path_node,
                                format!(
                                    "enumeration type \"{enum_name}\" does not contain the value \"{name}\""
                                ),
                            ))
                        } else {
                            None
                        }
                    }
                    ConstValue::String(name) => {
                        if !enum_values.contains_key(name.as_str()) {
                            Some(valid_error(
                                &path_node,
                                format!(
                                    "enumeration type \"{enum_name}\" does not contain the value \"{name}\""
                                ),
                            ))
                        } else {
                            None
                        }
                    }
                    _ => Some(valid_error(
                        &path_node,
                        format!("expected type \"{type_name}\""),
                    )),
                },
                registry::MetaType::InputObject(registry::InputObjectType {
                    input_fields,
                    name: object_name,
                    oneof,
                    ..
                }) => match value {
                    ConstValue::Object(values) => {
                        if *oneof {
                            if values.len() != 1 {
                                return Some(valid_error(
                                    &path_node,
                                    "oneOf input objects require exactly one field".to_string(),
                                ));
                            }

                            if let ConstValue::Null = values[0] {
                                return Some(valid_error(
                                    &path_node,
                                    "oneOf input objects require a non null argument".to_string(),
                                ));
                            }
                        }

                        let mut input_names: HashSet<&str> =
                            values.keys().map(AsRef::as_ref).collect::<HashSet<_>>();

                        for field in input_fields.values() {
                            input_names.remove::<str>(&field.name);
                            if let Some(value) = values.get::<str>(&field.name) {
                                if let Some(reason) = is_valid_input_value(
                                    registry,
                                    &field.ty,
                                    value,
                                    QueryPathNode {
                                        parent: Some(&path_node),
                                        segment: QueryPathSegment::Name(&field.name),
                                    },
                                ) {
                                    return Some(reason);
                                }
                            } else if registry::MetaTypeName::create(&field.ty).is_non_null()
                                && field.default_value.is_none()
                            {
                                return Some(valid_error(
                                    &path_node,
                                    format!(
                                        "field \"{}\" of type \"{object_name}\" is required but not provided",
                                        field.name,
                                    ),
                                ));
                            }
                        }

                        if let Some(name) = input_names.iter().next() {
                            return Some(valid_error(
                                &path_node,
                                format!("unknown field \"{name}\" of type \"{object_name}\""),
                            ));
                        }

                        None
                    }
                    _ => None,
                },
                _ => None,
            }
        }
    }
}
