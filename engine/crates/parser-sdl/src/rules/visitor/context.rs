use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::RwLock,
};

use common_types::UdfKind;
use engine::{
    model::{__Schema, __Type},
    registry::{ConnectorIdGenerator, MetaField, MetaInputValue, SchemaID, SchemaIDGenerator},
    AuthorizerProvider, LegacyOutputType, Registry, Schema,
};
use engine_parser::{
    types::{
        DirectiveDefinition, SchemaDefinition, ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
    },
    Pos, Positioned,
};
use engine_value::Name;
use registry_v1::MetaType;

use super::{warnings::Warnings, RuleError, TypeStackType, MUTATION_TYPE, QUERY_TYPE};
use crate::{
    federation::FederatedGraphConfig,
    rules::{
        codegen_directive::CodegenDirective,
        federation::{FederationVersion, KeyDirective},
        operation_limits_directive::OperationLimitsDirective,
        trusted_documents_directive::TrustedDocumentsDirective,
    },
    validations::validate_federation_keys,
    GlobalCacheRules, GlobalCacheTarget, GraphqlDirective, MongoDBDirective, OpenApiDirective, ParseResult,
    PostgresDirective,
};

/// The VisitorContext to visit every types from the Schema.
pub struct VisitorContext<'a> {
    pub(crate) directives: HashMap<String, &'a Positioned<DirectiveDefinition>>,
    pub(crate) types: HashMap<String, Cow<'a, Positioned<TypeDefinition>>>,
    #[allow(dead_code)]
    pub(crate) schema: Vec<&'a Positioned<SchemaDefinition>>,
    pub(crate) errors: Vec<RuleError>,
    pub(crate) warnings: Warnings,
    pub(crate) type_stack: TypeStackType<'a>,
    pub(crate) queries: Vec<MetaField>,
    pub(crate) mutations: Vec<MetaField>,
    pub schema_id_generator: SchemaIDGenerator,

    /// A generator used to generate unique identifiers for each connector present in the schema.
    ///
    /// This identifier is stable for the duration of the schema, but does not persist beyond
    /// schema generation. It can be used to pass along when referencing data stored within the
    /// schema (such as global headers), but *MUST NOT* be used for anything that requires a stable
    /// identifier across schema generations.
    pub connector_id_generator: ConnectorIdGenerator,

    /// Each schema to build should contains a SchemaID -> MetaType String to be
    /// able to construct the whole SchemaRegistry at the end of the parsing.
    pub schema_to_build: RwLock<HashMap<SchemaID, String>>,
    pub registry: RefCell<Registry>,
    pub variables: &'a HashMap<String, String>,
    pub(crate) required_resolvers: HashSet<String>,
    pub(crate) openapi_directives: Vec<(OpenApiDirective, Pos)>,
    pub(crate) graphql_directives: Vec<(GraphqlDirective, Pos)>,
    pub(crate) mongodb_directives: Vec<(MongoDBDirective, Pos)>,
    pub(crate) postgres_directives: Vec<(PostgresDirective, Pos)>,
    pub(crate) global_cache_rules: GlobalCacheRules<'static>,
    pub(crate) operation_limits_directive: Option<OperationLimitsDirective>,
    pub(crate) trusted_documents_directive: Option<TrustedDocumentsDirective>,
    pub(crate) codegen_directive: Option<CodegenDirective>,

    pub(crate) key_directives_to_validate: Vec<(Pos, KeyDirective, String)>,

    pub federation: Option<FederationVersion>,

    pub federated_graph_config: FederatedGraphConfig,
}

impl<'a> VisitorContext<'a> {
    #[cfg(test)] // Used in tests.
    pub(crate) fn new_for_tests(document: &'a ServiceDocument) -> Self {
        use std::sync::OnceLock;
        static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
        Self::new(document, MAP.get_or_init(HashMap::new))
    }

    pub(crate) fn new(document: &'a ServiceDocument, variables: &'a HashMap<String, String>) -> Self {
        let mut schema = Vec::new();
        let mut types = HashMap::new();
        let mut directives = HashMap::new();

        for type_def in &document.definitions {
            match type_def {
                TypeSystemDefinition::Type(ty) => {
                    types.insert(ty.node.name.node.to_string(), Cow::Borrowed(ty));
                }
                TypeSystemDefinition::Schema(schema_ty) => {
                    schema.push(schema_ty);
                }
                TypeSystemDefinition::Directive(directive) => {
                    directives.insert(directive.node.name.node.to_string(), directive);
                }
            }
        }

        // Built-in scalars
        add_fake_scalar(&mut types, "String");
        add_fake_scalar(&mut types, "ID");
        add_fake_scalar(&mut types, "Int");
        add_fake_scalar(&mut types, "Float");
        add_fake_scalar(&mut types, "Boolean");

        Self {
            directives,
            types,
            schema,
            errors: Default::default(),
            warnings: Default::default(),
            type_stack: Default::default(),
            registry: RefCell::new(Schema::create_registry()),
            mutations: Default::default(),
            queries: Default::default(),
            schema_to_build: Default::default(),
            connector_id_generator: Default::default(),
            schema_id_generator: Default::default(),
            variables,
            required_resolvers: Default::default(),
            openapi_directives: Vec::new(),
            graphql_directives: Vec::new(),
            mongodb_directives: Vec::new(),
            postgres_directives: Vec::new(),
            global_cache_rules: Default::default(),
            federation: None,
            federated_graph_config: Default::default(),
            operation_limits_directive: None,
            trusted_documents_directive: None,
            codegen_directive: None,
            key_directives_to_validate: vec![],
        }
    }

    pub(crate) fn validate(&mut self) {
        let mut key_errors = vec![];
        let registry = self.registry.borrow();
        for (pos, directive, type_name) in &self.key_directives_to_validate {
            let Some(MetaType::Object(object)) = registry.types.get(type_name) else {
                continue;
            };
            key_errors.extend(validate_federation_keys(*pos, directive, object))
        }
        self.errors.append(&mut key_errors)
    }

    /// Finish the Registry
    pub(crate) fn finish(mut self) -> ParseResult<'static> {
        let mut registry = self.registry.take();

        if self.federation.is_some() {
            registry.enable_federation = true;
        } else {
            // Clear out the federation entities if federation isn't enabled
            registry.federation_entities.clear();
        }

        if !self.mutations.is_empty() {
            registry.mutation_type = Some(MUTATION_TYPE.to_string());
        }

        registry.create_type(
            |registry| {
                let schema_type = __Schema::create_type_info(registry);
                let mut fields = Vec::with_capacity(self.queries.len() + 1);
                fields.push(MetaField {
                    name: "__schema".to_string(),
                    description: Some("Access the current type schema of this server.".to_string()),
                    ty: schema_type,
                    ..Default::default()
                });
                fields.push(MetaField {
                    name: "__type".to_string(),
                    args: [MetaInputValue::new("name", "String!")]
                        .into_iter()
                        .map(|value| (value.name.clone(), value))
                        .collect(),
                    description: Some("Access the current type schema of this server.".to_string()),
                    ty: __Type::create_type_info(registry),
                    ..Default::default()
                });
                fields.extend(self.queries);

                engine::registry::ObjectType::new(QUERY_TYPE.to_owned(), fields)
                    .with_cache_control(
                        self.global_cache_rules
                            .get(&GlobalCacheTarget::Type(Cow::Borrowed(QUERY_TYPE)))
                            .cloned()
                            .map(Box::new),
                    )
                    .into()
            },
            QUERY_TYPE,
            QUERY_TYPE,
        );

        if !self.mutations.is_empty() {
            registry.create_type(
                |_| engine::registry::ObjectType::new(MUTATION_TYPE.to_owned(), self.mutations).into(),
                MUTATION_TYPE,
                MUTATION_TYPE,
            );
        }

        if registry.enable_federation && registry.has_entities() {
            registry.create_federation_types();
        }

        registry.remove_unused_types();

        if let Some(directive) = self.codegen_directive.take() {
            registry.codegen = Some(directive.into())
        }

        registry.operation_limits = self
            .operation_limits_directive
            .take()
            .map(From::from)
            .unwrap_or_default();

        registry.trusted_documents = self.trusted_documents_directive.take().map(From::from);

        let mut required_udfs = self
            .required_resolvers
            .into_iter()
            .map(|udf_name| (UdfKind::Resolver, udf_name))
            .collect::<HashSet<_>>();
        if let Some(engine::AuthProvider::Authorizer(AuthorizerProvider { ref name })) =
            registry.auth.provider.as_deref()
        {
            required_udfs.insert((UdfKind::Authorizer, name.clone()));
        }

        self.federated_graph_config.operation_limits = registry.operation_limits.clone();

        // lets make sure we add the global rules to the federated graph config as well
        self.federated_graph_config.global_cache_rules = self.global_cache_rules.clone();

        ParseResult {
            global_cache_rules: self.global_cache_rules,
            federated_graph_config: registry.is_federated.then_some(self.federated_graph_config),
            registry,
            required_udfs,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn report_error<T: Into<String>>(&mut self, locations: Vec<Pos>, msg: T) {
        self.errors.push(RuleError::new(locations, msg));
    }

    #[allow(dead_code)]
    pub(crate) fn append_errors(&mut self, errors: Vec<RuleError>) {
        self.errors.extend(errors);
    }

    pub(crate) fn with_type<F: FnMut(&mut VisitorContext<'a>)>(&mut self, ty: Option<&'a Positioned<Type>>, mut f: F) {
        self.type_stack.push((ty, None));
        f(self);
        self.type_stack.pop();
    }

    pub(crate) fn with_definition_type<F: FnMut(&mut VisitorContext<'a>)>(
        &mut self,
        ty: Option<&'a Positioned<TypeDefinition>>,
        mut f: F,
    ) {
        self.type_stack.push((None, ty));
        f(self);
        self.type_stack.pop();
    }

    pub(crate) fn push_namespaced_query(&mut self, type_name: Option<&str>, meta_field: MetaField) {
        match type_name {
            Some(type_name) => self.push_namespaced_field(type_name, meta_field),
            None => self.queries.push(meta_field),
        }
    }

    pub(crate) fn push_namespaced_mutation(&mut self, type_name: Option<&str>, meta_field: MetaField) {
        match type_name {
            Some(type_name) => self.push_namespaced_field(type_name, meta_field),
            None => self.mutations.push(meta_field),
        }
    }

    fn push_namespaced_field(&mut self, type_name: &str, meta_field: MetaField) {
        let fields = self
            .registry
            .get_mut()
            .types
            .get_mut(type_name)
            .and_then(|r#type| r#type.fields_mut())
            .expect("Namespaced query/mutation type not registered.");

        fields.insert(meta_field.name.to_string(), meta_field);
    }
}

/// Add a fake scalar to the types HashMap if it isn't added by the schema.
fn add_fake_scalar(types: &mut HashMap<String, Cow<'_, Positioned<TypeDefinition>>>, name: &str) {
    match types.entry(name.to_string()) {
        Entry::Vacant(v) => {
            let pos = Positioned::new(
                TypeDefinition {
                    extend: false,
                    kind: TypeKind::Scalar,
                    name: Positioned::new(Name::new(name), Pos { line: 0, column: 0 }),
                    description: None,
                    directives: vec![],
                },
                Pos { line: 0, column: 0 },
            );

            v.insert(Cow::Owned(pos));
        }
        Entry::Occupied(_) => {}
    };
}
