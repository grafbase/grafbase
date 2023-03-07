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
    Enum, InputField, InputObject, OpenApiGraph, Operation, OutputType, PathParameter, QueryParameter, RequestBody,
    WrappingType,
};

pub fn output(graph: &OpenApiGraph, registry: &mut Registry) {
    register_scalars(registry);

    for output_type in graph.output_types() {
        let Some(metatype) = output_type.to_meta_type(graph) else { continue };

        registry.types.insert(metatype.name().to_string(), metatype);
    }

    for input_object in graph.input_objects() {
        let Some(metatype) = input_object.to_meta_type(graph) else { continue };

        registry.types.insert(metatype.name().to_string(), metatype);
    }

    for en in graph.enums() {
        let Some(metatype) = en.to_meta_type(graph) else { continue };

        registry.types.insert(metatype.name().to_string(), metatype);
    }

    let query_operations = graph.query_operations();
    if !query_operations.is_empty() {
        let query_fields = registry
            .query_root_mut()
            .fields_mut()
            .expect("QueryRoot to be an Object");

        for op in query_operations {
            let Some(metafield) = op.as_meta_field(graph) else { continue };
            query_fields.insert(metafield.name.clone(), metafield);
        }
    }

    let mutation_operations = graph.mutation_operations();
    if !mutation_operations.is_empty() {
        if registry.mutation_type.is_none() {
            registry.mutation_type = Some("Mutation".to_string());
            registry
                .types
                .insert("Mutation".to_string(), object("Mutation".to_string(), vec![]));
        }

        let mutation_fields = registry
            .mutation_root_mut()
            .fields_mut()
            .expect("MutationRoot to be an Object");

        for op in mutation_operations {
            let Some(metafield) = op.as_meta_field(graph) else { continue };

            mutation_fields.insert(metafield.name.clone(), metafield);
        }
    }

    registry.remove_unused_types();
}

impl OutputType {
    fn to_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        match self {
            OutputType::Object(_) => Some(object(
                name,
                self.fields(graph).into_iter().map(|field| field.to_meta_field()),
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

impl InputObject {
    fn to_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
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

pub struct Field {
    pub api_name: String,
    pub ty: FieldType,
}

impl Field {
    pub fn new(api_name: String, ty: FieldType) -> Self {
        Field { api_name, ty }
    }

    pub fn graphql_name(&self) -> String {
        self.api_name.to_camel_case()
    }

    fn to_meta_field(&self) -> MetaField {
        let name = self.graphql_name();

        let resolve = Some(Resolver {
            id: None,
            r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
                key: self.api_name.clone(),
            }),
        });

        MetaField {
            resolve,
            ..meta_field(name, self.ty.to_string())
        }
    }
}

pub enum FieldType {
    NonNull(Box<FieldType>),
    List(Box<FieldType>),
    Named(String),
}

impl FieldType {
    pub fn new(wrapping: &WrappingType, name: String) -> FieldType {
        match wrapping {
            WrappingType::NonNull(inner) => FieldType::NonNull(Box::new(FieldType::new(inner.as_ref(), name))),
            WrappingType::List(inner) => FieldType::List(Box::new(FieldType::new(inner.as_ref(), name))),
            WrappingType::Named => FieldType::Named(name),
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::NonNull(inner) => write!(f, "{inner}!"),
            FieldType::List(inner) => write!(f, "[{inner}]"),
            FieldType::Named(name) => write!(f, "{name}"),
        }
    }
}

impl Operation {
    fn as_meta_field(self, graph: &OpenApiGraph) -> Option<MetaField> {
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

        Some(MetaField {
            resolve: Some(Resolver {
                id: None,
                r#type: ResolverType::Http(HttpResolver {
                    method: "GET".to_string(),
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
                }),
            }),
            args,
            ..meta_field(self.name(graph)?.to_string(), self.ty(graph)?.to_string())
        })
    }
}

impl PathParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = self.input_value(graph)?;
        Some(MetaInputValue {
            name: self.name(graph)?.to_string(),
            description: None,
            ty: FieldType::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
            default_value: None,
            visible: None,
            validators: None,
            is_secret: false,
        })
    }
}

impl QueryParameter {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = self.input_value(graph)?;
        Some(MetaInputValue {
            name: self.name(graph)?.to_owned(),
            description: None,
            ty: FieldType::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
            default_value: None,
            visible: None,
            validators: None,
            is_secret: false,
        })
    }
}

impl RequestBody {
    fn to_meta_input_value(self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = self.input_value(graph)?;
        Some(MetaInputValue {
            name: self.argument_name().to_owned(),
            description: None,
            ty: FieldType::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
            default_value: None,
            visible: None,
            validators: None,
            is_secret: false,
        })
    }
}

impl InputField<'_> {
    fn to_meta_input_value(&self, graph: &OpenApiGraph) -> Option<MetaInputValue> {
        let input_value = &self.value_type;
        Some(MetaInputValue {
            name: self.name.to_string(),
            description: None,
            ty: FieldType::new(input_value.wrapping_type(), input_value.type_name(graph)?).to_string(),
            default_value: None,
            visible: None,
            validators: None,
            is_secret: false,
        })
    }
}

impl Enum {
    fn to_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        let values = self.values(graph)?;
        Some(MetaType::Enum {
            name: name.clone(),
            description: None,
            enum_values: values
                .iter()
                .map(|value| {
                    // TODO: Need to screaming snake case these somehow
                    (
                        value.clone(),
                        MetaEnumValue {
                            name: value.clone(),
                            description: None,
                            deprecation: NoDeprecated,
                            visible: None,
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
