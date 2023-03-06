use dynaql::{
    indexmap::IndexMap,
    registry::{
        resolvers::{
            context_data::ContextDataResolver,
            http::{self, HttpResolver},
            Resolver, ResolverType,
        },
        variables::VariableResolveDefinition,
        MetaField, MetaInputValue, MetaType, Registry,
    },
    CacheControl,
};
use inflector::Inflector;

use crate::graph::{OpenApiGraph, OutputType, PathParameter, QueryOperation, WrappingType};

pub fn output(graph: &OpenApiGraph, registry: &mut Registry) {
    register_scalars(registry);

    for output_type in graph.output_types() {
        let Some(metatype) = output_type.as_meta_type(graph) else { continue };

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

    registry.remove_unused_types();
}

impl OutputType {
    fn as_meta_type(self, graph: &OpenApiGraph) -> Option<MetaType> {
        let name = self.name(graph)?;
        match self {
            OutputType::Object(_) => Some(MetaType::Object {
                name: name.clone(),
                description: None,
                fields: self
                    .fields(graph)
                    .into_iter()
                    .map(|field| (field.graphql_name(), field.as_meta_field()))
                    .collect(),
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
            }),
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

    fn as_meta_field(&self) -> MetaField {
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

impl QueryOperation {
    fn as_meta_field(self, graph: &OpenApiGraph) -> Option<MetaField> {
        let path_parameters = self.path_parameters(graph);

        let mut args = IndexMap::new();
        args.extend(path_parameters.iter().map(|param| {
            let input_value = param.to_meta_input_value(graph).unwrap();
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
                            http::Parameter {
                                name: name.clone(),
                                variable_resolve_definition: VariableResolveDefinition::InputTypeName(name),
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
            ty: FieldType::new(input_value.wrapping_type(), input_value.name(graph)?).to_string(),
            default_value: None,
            visible: None,
            validators: None,
            is_secret: false,
        })
    }
}
fn meta_field(name: String, ty: String) -> MetaField {
    MetaField {
        name,
        description: None,
        args: IndexMap::new(),
        ty,
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
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
