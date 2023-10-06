use engine_value::{ConstValue, Name};
use indexmap::IndexMap;

use crate::{
    registry::{
        scalars::{DynamicScalar, PossibleScalar},
        InputValueType, MetaEnumValue, MetaInputValue, MetaType, MetaTypeName,
    },
    ContextField, Error, ServerResult,
};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum InputResolveMode {
    #[default]
    Default,
    ApplyConnectorTransforms,
}

pub fn resolve_input(
    ctx_field: &ContextField<'_>,
    arg_name: &str,
    meta_input_value: &MetaInputValue,
    value: Option<ConstValue>,
    mode: InputResolveMode,
) -> ServerResult<Option<ConstValue>> {
    // We do keep serde_json::Value::Null here contrary to resolver_input_inner
    // as it allows casting to either T or Option<T> later.
    resolve_maybe_absent_input(
        ResolveContext {
            ctx: ctx_field,
            path: PathNode::new(arg_name),
            ty: meta_input_value.ty.as_str(),
            allow_list_coercion: true,
            default_value: meta_input_value.default_value.as_ref(),
        },
        value,
        mode,
    )
    .map_err(|err| err.into_server_error(ctx_field.item.pos))
}

pub fn apply_input_transforms(
    ctx_field: &ContextField<'_>,
    arg_name: &str,
    value: ConstValue,
    ty: &InputValueType,
) -> ServerResult<ConstValue> {
    resolve_present_input(
        ResolveContext {
            ctx: ctx_field,
            path: PathNode::new(arg_name),
            ty: ty.to_string().as_str(),
            allow_list_coercion: true,
            default_value: None,
        },
        value,
        InputResolveMode::ApplyConnectorTransforms,
    )
    .map_err(|err| err.into_server_error(ctx_field.item.pos))
}

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

    fn to_vec(&self) -> Vec<String> {
        let mut previous = self.previous.map(PathNode::to_vec).unwrap_or_default();
        previous.push(self.name.to_string());
        previous
    }
}

struct ResolveContext<'a> {
    ctx: &'a ContextField<'a>,
    path: PathNode<'a>,
    /// Expected GraphQL type
    ty: &'a str,
    /// Whether we allow list coercion at this point:
    /// https://spec.graphql.org/October2021/#sec-List.Input-Coercion
    /// Most of time this will be true expect for:
    /// ty: [[Int]]  value: [1, 2, 3] => Error: Incorrect item value
    allow_list_coercion: bool,
    default_value: Option<&'a ConstValue>,
}

impl<'a> ResolveContext<'a> {
    fn with_input(&'a self, path: &'a str, input: &'a MetaInputValue) -> ResolveContext<'a> {
        ResolveContext {
            ctx: &self.ctx,
            path: self.path.with(path),
            ty: input.ty.as_str(),
            allow_list_coercion: true,
            default_value: input.default_value.as_ref(),
        }
    }

    fn input_error(self, expected: &str) -> Error {
        Error::new(format!("{expected} for {}", self.path.to_vec().join(".")))
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
        None => matches!(MetaTypeName::create(&rctx.ty), MetaTypeName::NonNull(_))
            .then_some(Err(rctx.input_error("Unexpected null value")))
            .transpose(),
    }
}

fn resolve_present_input(
    rctx: ResolveContext<'_>,
    value: ConstValue,
    mode: InputResolveMode,
) -> Result<ConstValue, Error> {
    match MetaTypeName::create(&rctx.ty) {
        MetaTypeName::NonNull(type_name) => {
            if matches!(value, ConstValue::Null) {
                return Err(rctx.input_error("Unexpected null value"));
            }
            resolve_present_input(ResolveContext { ty: type_name, ..rctx }, value, mode)
        }
        MetaTypeName::List(type_name) => {
            if matches!(value, ConstValue::Null) {
                return Ok(value);
            }
            if let ConstValue::List(list) = value {
                let rctx = ResolveContext {
                    ty: &type_name,
                    allow_list_coercion: list.len() <= 1,
                    default_value: None,
                    ..rctx
                };
                let mut arr = Vec::new();
                for (idx, element) in list.into_iter().enumerate() {
                    let path = idx.to_string();
                    let rctx = ResolveContext {
                        path: rctx.path.with(&path),
                        ..rctx
                    };
                    arr.push(resolve_present_input(rctx, element, mode)?);
                }
                Ok(ConstValue::List(arr))
            } else if rctx.allow_list_coercion {
                Ok(ConstValue::List(vec![resolve_present_input(
                    ResolveContext {
                        ty: &type_name,
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
                .ctx
                .schema_env
                .registry
                .types
                .get(type_name)
                .expect("Registry has already been validated")
            {
                MetaType::InputObject(input_object) => {
                    if let ConstValue::Object(mut fields) = value {
                        let mut map = IndexMap::with_capacity(fields.len());
                        for (name, meta_input_value) in &input_object.input_fields {
                            if let Some(field_value) = resolve_maybe_absent_input(
                                rctx.with_input(name, &meta_input_value),
                                fields.remove(&Name::new(name)),
                                mode,
                            )? {
                                let field_name = meta_input_value
                                    .rename
                                    .as_ref()
                                    .filter(|_| matches!(mode, InputResolveMode::ApplyConnectorTransforms))
                                    .unwrap_or(name);
                                map.insert(Name::new(field_name), field_value);
                            }
                        }
                        if input_object.oneof && map.len() != 1 {
                            return Err(
                                rctx.input_error(&format!("Expected exactly one fields (@oneof), got {}", map.len()))
                            );
                        }
                        Ok(ConstValue::Object(map))
                    } else {
                        Err(rctx.input_error("Expected an Object"))
                    }
                }
                MetaType::Enum(enum_type) => resolve_input_enum(rctx, value, &enum_type.enum_values, mode),
                // TODO: this conversion ConstValue -> serde_json -> ConstValue is sad...
                // we need an intermediate representation between the database & engine
                MetaType::Scalar { .. } => Ok(ConstValue::from_json(
                    PossibleScalar::parse(type_name, value).map_err(|err| Error::new(err.message()))?,
                )?),
                _ => Err(rctx.input_error(&format!("Internal Error: Unsupported input type {type_name}"))),
            }
        }
    }
}

fn resolve_input_enum(
    rctx: ResolveContext<'_>,
    value: ConstValue,
    values: &IndexMap<String, MetaEnumValue>,
    mode: InputResolveMode,
) -> Result<ConstValue, Error> {
    let str_value = match &value {
        ConstValue::Enum(name) => name.as_str(),
        ConstValue::String(string) => string.as_str(),
        _ => return Err(rctx.input_error(&format!("Expected an enum, not a {}", value.kind_str()))),
    };
    let meta_value = values
        .get(str_value)
        .ok_or_else(|| rctx.input_error("Unknown enum value: {name}"))?;

    if let (InputResolveMode::ApplyConnectorTransforms, Some(value)) = (mode, &meta_value.value) {
        return Ok(ConstValue::String(value.clone()));
    }

    Ok(value)
}
