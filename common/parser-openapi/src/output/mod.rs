mod namespacing;

use std::borrow::Cow;

use dynaql::{
    indexmap::IndexMap,
    registry::{
        resolvers::{
            context_data::ContextDataResolver,
            http::{self, HttpResolver},
            Resolver, ResolverType,
        },
        variables::VariableResolveDefinition,
        Deprecation::NoDeprecated,
        MetaEnumValue, MetaField, MetaInputValue, MetaType, Registry,
    },
    CacheControl,
};
use inflector::Inflector;

use crate::graph::{
    Enum, InputField, InputObject, OpenApiGraph, Operation, OutputField, OutputFieldType, OutputType, PathParameter,
    QueryParameter, RequestBody, WrappingType,
};

use self::namespacing::RegistryExt;

pub fn output(graph: &OpenApiGraph, registry: &mut Registry) {
    register_scalars(registry);

    registry.types.extend(types_to_metatypes(graph.output_types(), graph));
    registry.types.extend(types_to_metatypes(graph.input_objects(), graph));
    registry.types.extend(types_to_metatypes(graph.enums(), graph));

    let query_operations = graph.query_operations();
    if !query_operations.is_empty() {
        registry
            .query_fields_mut(&graph.metadata)
            .extend(operations_to_fields(query_operations, graph));
    }

    let mutation_operations = graph.mutation_operations();
    if !mutation_operations.is_empty() {
        registry
            .mutation_fields_mut(&graph.metadata)
            .extend(operations_to_fields(mutation_operations, graph));
    }

    registry.remove_unused_types();
}

fn operations_to_fields<'a>(
    operations: impl IntoIterator<Item = Operation> + 'a,
    graph: &'a OpenApiGraph,
) -> impl Iterator<Item = (String, MetaField)> + 'a {
    operations
        .into_iter()
        .filter_map(|op| op.into_meta_field(graph))
        .map(|meta_field| (meta_field.name.clone(), meta_field))
}

fn types_to_metatypes<'a, T>(
    types: impl IntoIterator<Item = T> + 'a,
    graph: &'a OpenApiGraph,
) -> impl Iterator<Item = (String, MetaType)> + 'a
where
    T: IntoMetaType,
{
    types
        .into_iter()
        .filter_map(|ty| ty.into_meta_type(graph))
        .map(|meta_type| (meta_type.name().to_string(), meta_type))
}

trait IntoMetaType {
    fn into_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType>;
}

impl IntoMetaType for OutputType {
    fn into_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        match self {
            OutputType::Object(_) => Some(object(
                name,
                self.fields(graph)
                    .into_iter()
                    .map(|field| field.into_meta_field(graph))
                    .collect::<Option<Vec<_>>>()?,
            )),
            OutputType::Union(_) => Some(MetaType::Union {
                name: name.clone(),
                description: None,
                possible_types: self
                    .possible_types(graph)
                    .into_iter()
                    .filter_map(|ty| ty.name(graph))
                    .collect(),
                visible: None,
                rust_typename: name,
            }),
        }
    }
}

impl IntoMetaType for InputObject {
    fn into_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;

        Some(MetaType::InputObject {
            name: name.clone(),
            description: None,
            input_fields: self
                .fields(graph)
                .into_iter()
                .filter_map(|field| {
                    let meta_input_value = field.to_meta_input_value(graph)?;
                    Some((meta_input_value.name.clone(), meta_input_value))
                })
                .collect(),
            visible: None,
            rust_typename: name,
            oneof: self.one_of(),
        })
    }
}

pub enum OutputFieldKind {
    Enum,

    // For now we only really care about enums here
    Other,
}

impl OutputField {
    fn into_meta_field(self, graph: &OpenApiGraph) -> Option<MetaField> {
        let api_name = &self.name;
        let graphql_name = api_name.to_camel_case();

        let mut resolver_type = ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
            key: api_name.to_string(),
        });

        if let OutputFieldKind::Enum = &self.ty.inner_kind(graph) {
            resolver_type = ResolverType::Composition(vec![
                resolver_type,
                ResolverType::ContextDataResolver(ContextDataResolver::RemoteEnum),
            ]);
        }

        let resolve = Some(Resolver {
            id: None,
            r#type: resolver_type,
        });

        Some(MetaField {
            resolve,
            ..meta_field(
                graphql_name,
                TypeDisplay::from_output_field_type(&self.ty, graph)?.to_string(),
            )
        })
    }
}

pub struct TypeDisplay<'a> {
    wrapping: &'a WrappingType,
    name: Cow<'a, str>,
}

impl<'a> TypeDisplay<'a> {
    pub fn new(wrapping: &'a WrappingType, name: String) -> TypeDisplay<'a> {
        TypeDisplay {
            wrapping,
            name: Cow::Owned(name),
        }
    }

    fn from_output_field_type(ty: &'a OutputFieldType, graph: &OpenApiGraph) -> Option<TypeDisplay<'a>> {
        Some(TypeDisplay::new(&ty.wrapping, ty.type_name(graph)?))
    }
}

impl std::fmt::Display for TypeDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.wrapping {
            WrappingType::NonNull(inner) => write!(
                f,
                "{}!",
                TypeDisplay {
                    wrapping: inner.as_ref(),
                    name: Cow::Borrowed(&self.name)
                }
            ),
            WrappingType::List(inner) => write!(
                f,
                "[{}]",
                TypeDisplay {
                    wrapping: inner.as_ref(),
                    name: Cow::Borrowed(&self.name)
                }
            ),
            WrappingType::Named => write!(f, "{}", self.name),
        }
    }
}

impl Operation {
    fn into_meta_field(self, graph: &OpenApiGraph) -> Option<MetaField> {
        let path_parameters = self.path_parameters(graph);
        let query_parameters = self.query_parameters(graph);
        let request_body = self.request_body(graph);

        let mut args = IndexMap::new();
        args.extend(path_parameters.iter().map(|param| {
            let input_value = param.to_meta_input_value(graph).unwrap();
            (input_value.name.clone(), input_value)
        }));
        args.extend(query_parameters.iter().map(|param| {
            let input_value = param.to_meta_input_value(graph).unwrap();
            (input_value.name.clone(), input_value)
        }));
        args.extend(request_body.iter().map(|body| {
            let input_value = body.to_meta_input_value(graph).unwrap();
            (input_value.name.clone(), input_value)
        }));

        let output_type = self.ty(graph)?;
        let type_string = TypeDisplay::from_output_field_type(&output_type, graph)?.to_string();

        Some(MetaField {
            resolve: Some(Resolver {
                id: None,
                r#type: ResolverType::Http(HttpResolver {
                    method: self.http_method(graph)?,
                    url: self.url(graph)?,
                    api_name: graph.metadata.name.clone(),
                    path_parameters: path_parameters
                        .iter()
                        .map(|param| {
                            let name = param.name(graph).unwrap().to_string();
                            http::PathParameter {
                                name: name.clone(),
                                variable_resolve_definition: VariableResolveDefinition::InputTypeName(name),
                            }
                        })
                        .collect(),
                    query_parameters: query_parameters
                        .iter()
                        .map(|param| {
                            let name = param.name(graph).unwrap().to_owned();
                            http::QueryParameter {
                                name: name.clone(),
                                variable_resolve_definition: VariableResolveDefinition::InputTypeName(name),
                                encoding_style: param.encoding_style(graph).unwrap(),
                            }
                        })
                        .collect(),
                    request_body: self.request_body(graph).map(|request_body| {
                        dynaql::registry::resolvers::http::RequestBody {
                            variable_resolve_definition: VariableResolveDefinition::InputTypeName(
                                request_body.argument_name().to_owned(),
                            ),
                            content_type: request_body.content_type(graph),
                        }
                    }),
                }),
            }),
            args,
            ..meta_field(self.name(graph)?.to_string(), type_string)
        })
    }
}

impl PathParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = self.input_value(graph)?;
        Some(MetaInputValue::new(
            self.name(graph)?.to_string(),
            TypeDisplay::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
        ))
    }
}

impl QueryParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = self.input_value(graph)?;
        Some(MetaInputValue::new(
            self.name(graph)?,
            TypeDisplay::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
        ))
    }
}

impl RequestBody {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = self.input_value(graph)?;
        Some(MetaInputValue::new(
            self.argument_name().to_owned(),
            TypeDisplay::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
        ))
    }
}

impl InputField<'_> {
    fn to_meta_input_value(&self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = &self.value_type;
        Some(MetaInputValue::new(
            self.name.to_string(),
            TypeDisplay::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
        ))
    }
}

impl IntoMetaType for Enum {
    fn into_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        let values = self.values(graph)?;
        Some(MetaType::Enum {
            name: name.clone(),
            description: None,
            enum_values: values
                .iter()
                .map(|value| {
                    let name = value.to_screaming_snake_case();
                    (
                        name.clone(),
                        MetaEnumValue {
                            name,
                            description: None,
                            deprecation: NoDeprecated,
                            visible: None,
                            value: Some(value.clone()),
                        },
                    )
                })
                .collect(),
            visible: None,
            rust_typename: name,
        })
    }
}

fn meta_field(name: String, ty: String) -> MetaField {
    MetaField {
        name,
        description: None,
        args: IndexMap::new(),
        ty,
        deprecation: NoDeprecated,
        cache_control: CacheControl {
            public: true,
            max_age: 0,
        },
        external: false,
        requires: None,
        provides: None,
        visible: None,
        compute_complexity: None,
        edges: vec![],
        relation: None,
        resolve: None,
        transformer: None,
        required_operation: None,
        auth: None,
        plan: None,
    }
}

fn object(name: String, fields: impl IntoIterator<Item = MetaField>) -> MetaType {
    MetaType::Object {
        name: name.clone(),
        description: None,
        fields: fields.into_iter().map(|field| (field.name.clone(), field)).collect(),
        cache_control: CacheControl {
            public: true,
            max_age: 0,
        },
        extends: false,
        keys: None,
        visible: None,
        is_subscription: false,
        is_node: false,
        rust_typename: name,
        constraints: vec![],
    }
}

fn register_scalars(registry: &mut Registry) {
    use dynaql::registry::scalars::{JSONScalar, SDLDefinitionScalar};

    registry.types.insert(
        JSONScalar::name().unwrap().to_string(),
        MetaType::Scalar {
            name: JSONScalar::name().unwrap().to_string(),
            description: JSONScalar::description().map(ToString::to_string),
            is_valid: None,
            visible: None,
            specified_by_url: JSONScalar::specified_by().map(ToString::to_string),
        },
    );
}
