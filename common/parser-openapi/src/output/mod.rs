mod discriminators;
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
};
use inflector::Inflector;

use crate::graph::{
    Enum, InputField, InputObject, InputValue, OpenApiGraph, Operation, OutputField, OutputFieldType, OutputType,
    PathParameter, QueryParameter, RequestBody, WrappingType,
};

use self::namespacing::RegistryExt;

pub fn output(graph: &OpenApiGraph, registry: &mut Registry) {
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
                discriminators: Some(self.discriminators(graph)),
            }),
            OutputType::ScalarWrapper(_) => {
                let scalar_name = self.inner_scalar_kind(graph)?.type_name();
                Some(object(
                    name,
                    vec![MetaField {
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
                                key: "data".into(),
                            }),
                        }),
                        ..meta_field("data".into(), scalar_name)
                    }],
                ))
            }
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
    Union,

    // For now we only really care about enums & unions here
    Other,
}

impl OutputField {
    fn graphql_name(&self, graph: &OpenApiGraph) -> String {
        let graphql_name = self.openapi_name.to_camel_case();
        if self.looks_like_nodes_field(graph) {
            return "nodes".to_string();
        }
        graphql_name
    }

    fn into_meta_field(self, graph: &OpenApiGraph) -> Option<MetaField> {
        let api_name = &self.openapi_name;
        let graphql_name = self.graphql_name(graph);

        let mut resolver_type = ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
            key: api_name.to_string(),
        });

        resolver_type = resolver_type.and_then_maybe(self.ty.transforming_resolver(graph));

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

impl OutputFieldType {
    /// Some output types require transformation between their remote representation and their
    /// GraphQL representation.  This function returns an appropriate resolver to do that.
    pub fn transforming_resolver(&self, graph: &OpenApiGraph) -> Option<ResolverType> {
        match self.inner_kind(graph) {
            OutputFieldKind::Enum => Some(ResolverType::ContextDataResolver(ContextDataResolver::RemoteEnum)),
            OutputFieldKind::Union => Some(ResolverType::ContextDataResolver(ContextDataResolver::RemoteUnion)),
            OutputFieldKind::Other => None,
        }
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
                r#type: self
                    .http_resolver(graph, path_parameters, query_parameters)?
                    .and_then_maybe(output_type.transforming_resolver(graph)),
            }),
            args,
            ..meta_field(self.name(graph)?.to_string(), type_string)
        })
    }

    fn http_resolver(
        self,
        graph: &OpenApiGraph,
        path_parameters: Vec<PathParameter>,
        query_parameters: Vec<QueryParameter>,
    ) -> Option<ResolverType> {
        Some(ResolverType::Http(HttpResolver {
            method: self.http_method(graph),
            url: self.url(graph),
            api_name: graph.metadata.name.clone(),
            expected_status: self.expected_status(graph)?,
            path_parameters: path_parameters
                .iter()
                .map(|param| {
                    let name = param.openapi_name(graph).unwrap().to_string();
                    let input_name = param.graphql_name(graph).unwrap().to_string();
                    http::PathParameter {
                        name,
                        variable_resolve_definition: VariableResolveDefinition::InputTypeName(input_name),
                    }
                })
                .collect(),
            query_parameters: query_parameters
                .iter()
                .map(|param| {
                    let name = param.openapi_name(graph).unwrap().to_string();
                    let input_name = param.graphql_name(graph).unwrap().to_string();
                    http::QueryParameter {
                        name,
                        variable_resolve_definition: VariableResolveDefinition::InputTypeName(input_name),
                        encoding_style: param.encoding_style(graph).unwrap(),
                    }
                })
                .collect(),
            request_body: self
                .request_body(graph)
                .map(|request_body| dynaql::registry::resolvers::http::RequestBody {
                    variable_resolve_definition: VariableResolveDefinition::InputTypeName(
                        request_body.argument_name().to_owned(),
                    ),
                    content_type: request_body.content_type(graph).clone(),
                }),
        }))
    }
}

impl PathParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        self.input_value(graph)?
            .to_meta_input_value(&self.graphql_name(graph)?.to_string(), graph)
    }
}

impl QueryParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        self.input_value(graph)?
            .to_meta_input_value(&self.graphql_name(graph)?.to_string(), graph)
    }
}

impl RequestBody {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        self.input_value(graph)?
            .to_meta_input_value(self.argument_name(), graph)
    }
}

impl InputField<'_> {
    fn to_meta_input_value(&self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        self.value_type.to_meta_input_value(&self.name.to_string(), graph)
    }
}

impl InputValue {
    fn to_meta_input_value(&self, name: &str, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let mut graphql_value = MetaInputValue::new(
            name.to_string(),
            TypeDisplay::new(self.wrapping_type(), self.type_name(graph)?).to_string(),
        );
        graphql_value.default_value = self.default_value(graph);

        Some(graphql_value)
    }
}

impl IntoMetaType for Enum {
    fn into_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        let values = self.values(graph);
        Some(MetaType::Enum {
            name: name.clone(),
            description: None,
            enum_values: values
                .into_iter()
                .map(|value| {
                    let name = value.to_screaming_snake_case();
                    (
                        name.clone(),
                        MetaEnumValue {
                            name,
                            description: None,
                            deprecation: NoDeprecated,
                            visible: None,
                            value: Some(value.to_string()),
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
        cache_control: Default::default(),
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
        cache_control: Default::default(),
        extends: false,
        keys: None,
        visible: None,
        is_subscription: false,
        is_node: false,
        rust_typename: name,
        constraints: vec![],
    }
}

trait ResolverTypeExt {
    fn and_then(self, other_resolver: ResolverType) -> Self;
    fn and_then_maybe(self, other_resolver: Option<ResolverType>) -> Self;
}

impl ResolverTypeExt for ResolverType {
    fn and_then(mut self, other_resolver: ResolverType) -> Self {
        match &mut self {
            ResolverType::Composition(resolvers) => {
                resolvers.push(other_resolver);
                self
            }
            _ => ResolverType::Composition(vec![self, other_resolver]),
        }
    }

    fn and_then_maybe(self, other_resolver: Option<ResolverType>) -> Self {
        match other_resolver {
            Some(other) => self.and_then(other),
            None => self,
        }
    }
}
