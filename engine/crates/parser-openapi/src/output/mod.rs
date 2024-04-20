mod discriminators;
mod federation;
mod namespacing;

use std::borrow::Cow;

use engine::{
    indexmap::IndexMap,
    registry::{
        resolvers::{
            http::{self, HttpResolver},
            transformer::Transformer,
            Resolver,
        },
        variables::VariableResolveDefinition,
        EnumType, InputObjectType, InputValueType, MetaEnumValue, MetaField, MetaInputValue, MetaType, ObjectType,
        Registry, UnionType,
    },
};
use inflector::Inflector;

use self::namespacing::RegistryExt;
use crate::graph::{
    Enum, InputField, InputObject, InputValue, OpenApiGraph, Operation, OutputField, OutputFieldType, OutputType,
    PathParameter, QueryParameter, RequestBody, WrappingType,
};

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

    registry.federation_entities = federation::federation_entities(graph);

    registry.remove_unused_types();
}

fn operations_to_fields<'a>(
    operations: impl IntoIterator<Item = Operation> + 'a,
    graph: &'a OpenApiGraph,
) -> impl Iterator<Item = (String, MetaField)> + 'a {
    operations
        .into_iter()
        .filter_map(|op| {
            let metafield = op.into_meta_field(graph);
            if metafield.is_none() {
                tracing::info!("Couldn't generate a MetaField for {:?}, skipping", op.name(graph));
            }
            metafield
        })
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
            OutputType::Union(_) => Some(MetaType::Union(UnionType {
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
            })),
            OutputType::ScalarWrapper(_) => {
                let scalar_name = self.inner_scalar_kind(graph)?.type_name();
                Some(object(
                    name,
                    vec![MetaField {
                        resolver: Transformer::select("data").into(),
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

        Some(
            InputObjectType::new(
                name,
                self.fields(graph)
                    .into_iter()
                    .filter_map(|field| field.to_meta_input_value(graph)),
            )
            .with_oneof(self.one_of())
            .into(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFieldKind {
    Scalar,
    ScalarWrapper,
    Object,
    Enum,
    Union,
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

        let resolver: Resolver = Transformer::select(api_name).into();
        let resolver = resolver.and_then_maybe(self.ty.transforming_resolver(graph));

        Some(MetaField {
            resolver,
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
    pub fn transforming_resolver(&self, graph: &OpenApiGraph) -> Option<Resolver> {
        match self.inner_kind(graph) {
            OutputFieldKind::Enum => Some(Resolver::Transformer(Transformer::RemoteEnum)),
            OutputFieldKind::Union => Some(Resolver::Transformer(Transformer::RemoteUnion)),
            _ => None,
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
        args.extend(path_parameters.iter().filter_map(|param| {
            let input_value = param.to_meta_input_value(graph)?;
            Some((input_value.name.clone(), input_value))
        }));
        args.extend(query_parameters.iter().filter_map(|param| {
            let input_value = param.to_meta_input_value(graph)?;
            Some((input_value.name.clone(), input_value))
        }));
        args.extend(request_body.iter().filter_map(|body| {
            let input_value = body.to_meta_input_value(graph)?;
            Some((input_value.name.clone(), input_value))
        }));

        let mut output_type = self.ty(graph)?;

        // HTTP requests can fail so it's best if we make Operation fields
        // optional regardless of what the API says. This avoids errors
        // bubbling further up the query heirarchy.
        output_type.wrapping = output_type.wrapping.unwrap_required();

        let type_string = TypeDisplay::from_output_field_type(&output_type, graph)?.to_string();

        Some(MetaField {
            resolver: self
                .http_resolver(
                    graph,
                    self.http_path_parameters(graph),
                    self.http_query_parameters(graph),
                )?
                .and_then_maybe(output_type.transforming_resolver(graph)),
            args,
            ..meta_field(self.name(graph)?.to_string(), type_string)
        })
    }

    fn http_path_parameters(self, graph: &OpenApiGraph) -> Vec<http::PathParameter> {
        self.path_parameters(graph)
            .iter()
            .map(|param| {
                let name = param.openapi_name(graph).to_string();
                let input_name = param.graphql_name(graph).to_string();
                http::PathParameter {
                    name,
                    variable_resolve_definition: VariableResolveDefinition::connector_input_type_name(input_name),
                }
            })
            .collect()
    }

    fn http_query_parameters(self, graph: &OpenApiGraph) -> Vec<http::QueryParameter> {
        self.query_parameters(graph)
            .iter()
            .map(|param| {
                let name = param.openapi_name(graph).to_string();
                let input_name = param.graphql_name(graph).to_string();
                http::QueryParameter {
                    name,
                    variable_resolve_definition: VariableResolveDefinition::connector_input_type_name(input_name),
                    encoding_style: param.encoding_style(graph).unwrap(),
                }
            })
            .collect()
    }

    fn http_resolver(
        self,
        graph: &OpenApiGraph,
        path_parameters: Vec<http::PathParameter>,
        query_parameters: Vec<http::QueryParameter>,
    ) -> Option<Resolver> {
        Some(Resolver::Http(Box::new(HttpResolver {
            method: self.http_method(graph),
            url: self.url(graph),
            api_name: graph.metadata.unique_namespace(),
            expected_status: self.expected_status(graph)?,
            path_parameters,
            query_parameters,
            request_body: self
                .request_body(graph)
                .map(|request_body| engine::registry::resolvers::http::RequestBody {
                    variable_resolve_definition: VariableResolveDefinition::connector_input_type_name(
                        request_body.argument_name().to_owned(),
                    ),
                    content_type: request_body.content_type(graph).clone(),
                }),
        })))
    }
}

impl PathParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        self.input_value(graph)?
            .to_meta_input_value(&self.graphql_name(graph).to_string(), graph)
    }
}

impl QueryParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        self.input_value(graph)?
            .to_meta_input_value(&self.graphql_name(graph).to_string(), graph)
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
        let graphql_name = self.name.to_string();
        let openapi_name = self.name.openapi_name();
        let meta_input_value = self
            .value_type
            .to_meta_input_value(&graphql_name, graph)?
            .with_rename((openapi_name != graphql_name).then(|| openapi_name.to_string()));

        Some(meta_input_value)
    }
}

impl InputValue {
    fn to_meta_input_value(&self, name: &str, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let mut graphql_value = MetaInputValue::new(name.to_string(), self.to_input_value_type(graph)?);
        graphql_value.default_value = self.default_value(graph);

        Some(graphql_value)
    }

    fn to_input_value_type(&self, graph: &OpenApiGraph) -> Option<InputValueType> {
        Some(
            TypeDisplay::new(self.wrapping_type(), self.type_name(graph)?)
                .to_string()
                .into(),
        )
    }
}

impl IntoMetaType for Enum {
    fn into_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        let values = self.values(graph);
        Some(
            EnumType::new(
                name,
                values.into_iter().map(|value| {
                    let graphql_value = value.to_screaming_snake_case();
                    MetaEnumValue {
                        value: Some(value.to_string()),
                        ..MetaEnumValue::new(graphql_value)
                    }
                }),
            )
            .into(),
        )
    }
}

fn meta_field(name: String, ty: String) -> MetaField {
    MetaField {
        name,
        args: IndexMap::new(),
        ty: ty.into(),
        ..Default::default()
    }
}

fn object(name: String, fields: impl IntoIterator<Item = MetaField>) -> MetaType {
    ObjectType::new(name, fields).into()
}
