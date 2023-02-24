use dynaql::{
    indexmap::IndexMap,
    registry::{
        resolvers::{context_data::ContextDataResolver, http::HttpResolver, Resolver, ResolverType},
        MetaField, MetaType, Registry,
    },
    CacheControl,
};
use inflector::Inflector;

use crate::graph::{OpenApiGraph, OutputType, QueryOperation, WrappingType};

pub fn output(graph: &OpenApiGraph, registry: &mut Registry) {
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
        Some(MetaField {
            resolve: Some(Resolver {
                id: None,
                r#type: ResolverType::Http(HttpResolver {
                    method: "GET".to_string(),
                    url: self.url(graph)?.to_string(),
                    api_name: graph.metadata.name.clone(),
                }),
            }),
            ..meta_field(self.name(graph)?.to_string(), self.ty(graph)?.to_string())
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
