use std::collections::HashSet;

use engine_scalars::{DynamicScalar, PossibleScalar};
use engine_value::{ConstValue, Value};
use meta_type_name::MetaTypeName;
use query_path::QueryPath;
use registry_v2::{MetaType, ScalarParser};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Scope<'a> {
    Operation(Option<&'a str>),
    Fragment(&'a str),
}

fn valid_error(path_node: &QueryPath, msg: String) -> String {
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
        Value::List(values) => values.iter().for_each(|value| referenced_variables_to_vec(value, vars)),
        Value::Object(obj) => obj.values().for_each(|value| referenced_variables_to_vec(value, vars)),
        _ => {}
    }
}

pub fn is_valid_input_value(
    registry: &registry_v2::Registry,
    type_name: &str,
    value: &ConstValue,
    path: QueryPath,
) -> Option<String> {
    match MetaTypeName::create(type_name) {
        MetaTypeName::NonNull(type_name) => match value {
            ConstValue::Null => Some(valid_error(
                &path,
                format!("expected type \"{type_name}\" but found null"),
            )),
            _ => is_valid_input_value(registry, type_name, value, path),
        },
        MetaTypeName::List(type_name) => match value {
            ConstValue::List(elems) => elems
                .iter()
                .enumerate()
                .find_map(|(idx, elem)| is_valid_input_value(registry, type_name, elem, path.clone().child(idx))),
            ConstValue::Null => None,
            _ => is_valid_input_value(registry, type_name, value, path),
        },
        MetaTypeName::Named(type_name) => {
            if let ConstValue::Null = value {
                return None;
            }

            match registry
                .lookup_type(type_name)
                .unwrap_or_else(|| panic!("{type_name} unknown"))
            {
                MetaType::Scalar(scalar) => match scalar.parser() {
                    ScalarParser::PassThrough => None,
                    ScalarParser::BestEffort => {
                        if PossibleScalar::is_valid(type_name, value) {
                            None
                        } else {
                            Some(valid_error(&path, format!("expected type \"{type_name}\"")))
                        }
                    }
                },
                MetaType::Enum(enum_type) => {
                    let enum_name = enum_type.name();
                    match value {
                        ConstValue::Enum(name) => {
                            if enum_type.value(name.as_str()).is_none() {
                                Some(valid_error(
                                    &path,
                                    format!("enumeration type \"{enum_name}\" does not contain the value \"{name}\""),
                                ))
                            } else {
                                None
                            }
                        }
                        ConstValue::String(name) => {
                            if enum_type.value(name.as_str()).is_none() {
                                Some(valid_error(
                                    &path,
                                    format!("enumeration type \"{enum_name}\" does not contain the value \"{name}\""),
                                ))
                            } else {
                                None
                            }
                        }
                        _ => Some(valid_error(
                            &path,
                            format!("expected type \"{type_name}\" but got {value}"),
                        )),
                    }
                }
                MetaType::InputObject(input_object) => match value {
                    ConstValue::Object(values) => {
                        if input_object.oneof() {
                            if values.len() != 1 {
                                return Some(valid_error(
                                    &path,
                                    "oneOf input objects require exactly one field".to_string(),
                                ));
                            }

                            if let ConstValue::Null = values[0] {
                                return Some(valid_error(
                                    &path,
                                    "oneOf input objects require a non null argument".to_string(),
                                ));
                            }
                        }

                        let mut input_names: HashSet<&str> = values.keys().map(AsRef::as_ref).collect::<HashSet<_>>();

                        for field in input_object.input_fields() {
                            input_names.remove::<str>(field.name());
                            if let Some(value) = values.get::<str>(field.name()) {
                                if let Some(reason) = is_valid_input_value(
                                    registry,
                                    &field.ty().to_string(),
                                    value,
                                    path.clone().child(field.name()),
                                ) {
                                    return Some(reason);
                                }
                            } else if field.ty().is_non_null() && field.default_value().is_none() {
                                return Some(valid_error(
                                    &path,
                                    format!(
                                        "field \"{}\" of type \"{}\" is required but not provided",
                                        field.name(),
                                        input_object.name()
                                    ),
                                ));
                            }
                        }

                        if let Some(name) = input_names.iter().next() {
                            return Some(valid_error(
                                &path,
                                format!("unknown field \"{name}\" of type \"{}\"", input_object.name()),
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
