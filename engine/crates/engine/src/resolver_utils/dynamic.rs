use std::borrow::{Borrow, Cow};

use engine_parser::Pos;
use engine_scalars::{DynamicScalar, PossibleScalar};
use engine_value::{ConstValue, Name};
use indexmap::IndexMap;

use meta_type_name::MetaTypeName;
use registry_v2::{EnumType, MetaInputValue, MetaType};

use crate::{registry::InputValueType, Error, ServerResult};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum InputResolveMode {
    #[default]
    Default,
    ApplyConnectorTransforms,
}

pub fn resolve_input(
    registry: &registry_v2::Registry,
    error_pos: Pos,
    arg_name: &str,
    meta_input_value: MetaInputValue<'_>,
    value: Option<ConstValue>,
    mode: InputResolveMode,
) -> ServerResult<Option<ConstValue>> {
    // We do keep serde_json::Value::Null here contrary to resolver_input_inner
    // as it allows casting to either T or Option<T> later.
    let ty = meta_input_value.ty().to_string();
    resolve_maybe_absent_input(
        ResolveContext {
            registry,
            path: PathNode::new(arg_name),
            ty: Cow::Borrowed(&ty),
            allow_list_coercion: true,
            default_value: meta_input_value.default_value(),
        },
        value,
        mode,
    )
    .map_err(|err| err.into_server_error(error_pos))
}

pub fn apply_input_transforms(
    registry: &registry_v2::Registry,
    error_pos: Pos,
    arg_name: &str,
    value: ConstValue,
    ty: &InputValueType,
) -> ServerResult<ConstValue> {
    resolve_present_input(
        ResolveContext {
            registry,
            path: PathNode::new(arg_name),
            ty: Cow::Owned(ty.to_string()),
            allow_list_coercion: true,
            default_value: None,
        },
        value,
        InputResolveMode::ApplyConnectorTransforms,
    )
    .map_err(|err| err.into_server_error(error_pos))
}

#[derive(Clone, Copy)]
struct PathNode<'a> {
    name: &'a str,
    previous: Option<&'a PathNode<'a>>,
}

impl<'a> PathNode<'a> {
    fn new(name: &'a str) -> PathNode<'a> {
        PathNode { name, previous: None }
    }

    fn with(&'a self, name: &'a str) -> PathNode<'a> {
        PathNode {
            name,
            previous: Some(self),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    fn into_vec(&self) -> Vec<String> {
        let mut previous = self.previous.map(PathNode::into_vec).unwrap_or_default();
        previous.push(self.name.to_string());
        previous
    }
}

#[derive(Clone)]
struct ResolveContext<'a> {
    registry: &'a registry_v2::Registry,
    path: PathNode<'a>,
    /// Expected GraphQL type
    ty: Cow<'a, str>,
    /// Whether we allow list coercion at this point:
    /// https://spec.graphql.org/October2021/#sec-List.Input-Coercion
    /// Most of time this will be true expect for:
    /// ty: [[Int]]  value: [1, 2, 3] => Error: Incorrect item value
    allow_list_coercion: bool,
    default_value: Option<&'a ConstValue>,
}

impl<'a> ResolveContext<'a> {
    fn with_input(&'a self, path: &'a str, input: MetaInputValue<'a>) -> ResolveContext<'a> {
        ResolveContext {
            registry: self.registry,
            path: self.path.with(path),
            ty: Cow::Owned(input.ty().to_string()),
            allow_list_coercion: true,
            default_value: input.default_value(),
        }
    }

    fn input_error(self, expected: &str) -> Error {
        Error::new(format!("{expected} for {}", self.path.into_vec().join(".")))
    }
}

fn resolve_maybe_absent_input(
    rctx: ResolveContext<'_>,
    value: Option<ConstValue>,
    mode: InputResolveMode,
) -> Result<Option<ConstValue>, Error> {
    // Propagating default value to apply transforms (enum). But this was also done even before, I don't
    // remember exactly why though...
    match value.or_else(|| rctx.default_value.cloned()) {
        Some(value) => resolve_present_input(rctx, value, mode).map(Some),
        None => matches!(MetaTypeName::create(rctx.ty.borrow()), MetaTypeName::NonNull(_))
            .then_some(Err(rctx.input_error("Unexpected null value")))
            .transpose(),
    }
}

fn resolve_present_input(
    rctx: ResolveContext<'_>,
    value: ConstValue,
    mode: InputResolveMode,
) -> Result<ConstValue, Error> {
    match MetaTypeName::create(rctx.ty.borrow()) {
        MetaTypeName::NonNull(type_name) => {
            if matches!(value, ConstValue::Null) {
                return Err(rctx.input_error("Unexpected null value"));
            }
            resolve_present_input(
                ResolveContext {
                    ty: Cow::Borrowed(type_name),
                    ..rctx
                },
                value,
                mode,
            )
        }
        MetaTypeName::List(type_name) => {
            if matches!(value, ConstValue::Null) {
                return Ok(value);
            }
            if let ConstValue::List(list) = value {
                let rctx = ResolveContext {
                    ty: Cow::Borrowed(type_name),
                    allow_list_coercion: list.len() <= 1,
                    default_value: None,
                    ..rctx
                };
                let mut arr = Vec::new();
                for (idx, element) in list.into_iter().enumerate() {
                    let path = idx.to_string();
                    let rctx = ResolveContext {
                        path: rctx.path.with(&path),
                        ..rctx.clone()
                    };
                    arr.push(resolve_present_input(rctx, element, mode)?);
                }
                Ok(ConstValue::List(arr))
            } else if rctx.allow_list_coercion {
                Ok(ConstValue::List(vec![resolve_present_input(
                    ResolveContext {
                        ty: Cow::Borrowed(type_name),
                        allow_list_coercion: true,
                        default_value: None,
                        ..rctx
                    },
                    value,
                    mode,
                )?]))
            } else {
                Err(rctx.input_error("Expected a List"))
            }
        }
        MetaTypeName::Named(type_name) => {
            if matches!(value, ConstValue::Null) {
                return Ok(value);
            }
            match rctx
                .registry
                .lookup_type(type_name)
                .expect("Registry has already been validated")
            {
                MetaType::InputObject(input_object) => {
                    if let ConstValue::Object(mut fields) = value {
                        let mut map = IndexMap::with_capacity(fields.len());
                        for meta_input_value in input_object.input_fields() {
                            if let Some(field_value) = resolve_maybe_absent_input(
                                rctx.with_input(meta_input_value.name(), meta_input_value),
                                fields.shift_remove(&Name::new(meta_input_value.name())),
                                mode,
                            )? {
                                let field_name = meta_input_value
                                    .rename()
                                    .filter(|_| matches!(mode, InputResolveMode::ApplyConnectorTransforms))
                                    .unwrap_or(meta_input_value.name());
                                map.insert(Name::new(field_name), field_value);
                            }
                        }
                        if input_object.oneof() && map.len() != 1 {
                            return Err(
                                rctx.input_error(&format!("Expected exactly one fields (@oneof), got {}", map.len()))
                            );
                        }
                        Ok(ConstValue::Object(map))
                    } else {
                        Err(rctx.input_error("Expected an Object"))
                    }
                }
                MetaType::Enum(enum_type) => resolve_input_enum(rctx, value, enum_type, mode),
                // TODO: this conversion ConstValue -> serde_json -> ConstValue is sad...
                // we need an intermediate representation between the database & engine
                MetaType::Scalar { .. } => Ok(ConstValue::from_json(
                    PossibleScalar::parse(type_name, value).map_err(|err| Error::new(err.message()))?,
                )?),
                _ => Err(rctx
                    .clone()
                    .input_error(&format!("Internal Error: Unsupported input type {type_name}"))),
            }
        }
    }
}

fn resolve_input_enum(
    rctx: ResolveContext<'_>,
    value: ConstValue,
    ty: EnumType<'_>,
    mode: InputResolveMode,
) -> Result<ConstValue, Error> {
    let str_value = match &value {
        ConstValue::Enum(name) => name.as_str(),
        ConstValue::String(string) => string.as_str(),
        _ => return Err(rctx.input_error(&format!("Expected an enum, not a {}", value.kind_str()))),
    };
    let meta_value = ty
        .value(str_value)
        .ok_or_else(|| rctx.input_error("Unknown enum value: {name}"))?;

    if let (InputResolveMode::ApplyConnectorTransforms, Some(value)) = (mode, meta_value.value()) {
        return Ok(ConstValue::String(value.to_string()));
    }

    Ok(ConstValue::Enum(Name::new(str_value)))
}
