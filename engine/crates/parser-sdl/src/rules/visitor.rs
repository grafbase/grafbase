use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap, HashSet},
    fmt::{self, Display, Formatter},
    sync::RwLock,
};

use common_types::UdfKind;
use engine::{
    indexmap::IndexMap,
    model::{__Schema, __Type},
    registry::{
        relations::MetaRelation, ConnectorIdGenerator, MetaField, MetaInputValue, Registry, SchemaID, SchemaIDGenerator,
    },
    AuthorizerProvider, LegacyOutputType, Name, Pos, Positioned, Schema,
};
use engine_parser::types::{
    ConstDirective, DirectiveDefinition, FieldDefinition, InputValueDefinition, ObjectType, SchemaDefinition,
    ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use engine_value::ConstValue;

use super::{
    federation_directive::FederationVersion, graphql_directive::GraphqlDirective, openapi_directive::OpenApiDirective,
    postgresql_directive::PostgresDirective,
};
use crate::{
    rules::cache_directive::global::{GlobalCacheRules, GlobalCacheTarget},
    MongoDBDirective, ParseResult,
};

type TypeStackType<'a> = Vec<(Option<&'a Positioned<Type>>, Option<&'a Positioned<TypeDefinition>>)>;

pub const QUERY_TYPE: &str = "Query";
pub const MUTATION_TYPE: &str = "Mutation";

/// The VisitorContext to visit every types from the Schema.
pub struct VisitorContext<'a> {
    pub(crate) directives: HashMap<String, &'a Positioned<DirectiveDefinition>>,
    pub(crate) types: HashMap<String, Cow<'a, Positioned<TypeDefinition>>>,
    #[allow(dead_code)]
    pub(crate) schema: Vec<&'a Positioned<SchemaDefinition>>,
    pub(crate) errors: Vec<RuleError>,
    pub(crate) type_stack: TypeStackType<'a>,
    pub(crate) queries: Vec<MetaField>,
    pub(crate) mutations: Vec<MetaField>,
    /// Relations by name
    pub(crate) relations: IndexMap<String, MetaRelation>,
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
    pub(crate) postgresql_directives: Vec<(PostgresDirective, Pos)>,
    pub(crate) global_cache_rules: GlobalCacheRules<'static>,

    pub database_models_enabled: bool,
    pub federation: Option<FederationVersion>,
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

impl<'a> VisitorContext<'a> {
    #[cfg(test)] // Used in tests.
    pub(crate) fn new_for_tests(document: &'a ServiceDocument) -> Self {
        lazy_static::lazy_static! {
            static ref EMPTY_HASHMAP: HashMap<String, String> = HashMap::new();
        }
        Self::new(document, true, &EMPTY_HASHMAP)
    }

    /// Create a new unique [`SchemaID`] for this [`VisitorContext`] if the provided `ty` doesn't
    /// already have a [`SchemaID`]
    pub(crate) fn get_schema_id<S: AsRef<str>>(&self, ty: S) -> SchemaID {
        if let Some((id, _val)) = self
            .schema_to_build
            .try_read()
            .expect("Poisoned")
            .iter()
            .find(|(_id, val)| val.as_str() == ty.as_ref())
        {
            return *id;
        }
        let new_id = self.schema_id_generator.new_id();
        self.schema_to_build
            .try_write()
            .expect("Poisoned")
            .insert(new_id, ty.as_ref().to_string());
        new_id
    }

    pub(crate) fn new(
        document: &'a ServiceDocument,
        database_models_enabled: bool,
        variables: &'a HashMap<String, String>,
    ) -> Self {
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
            type_stack: Default::default(),
            registry: RefCell::new(Schema::create_registry()),
            mutations: Default::default(),
            queries: Default::default(),
            relations: Default::default(),
            schema_to_build: Default::default(),
            connector_id_generator: Default::default(),
            schema_id_generator: Default::default(),
            variables,
            required_resolvers: Default::default(),
            openapi_directives: Vec::new(),
            graphql_directives: Vec::new(),
            mongodb_directives: Vec::new(),
            postgresql_directives: Vec::new(),
            global_cache_rules: Default::default(),
            database_models_enabled,
            federation: None,
        }
    }

    /// Finish the Registry
    pub(crate) fn finish(self) -> ParseResult<'static> {
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
                            .unwrap_or_default(),
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

        registry.remove_unused_types();

        let mut required_udfs = self
            .required_resolvers
            .into_iter()
            .map(|udf_name| (UdfKind::Resolver, udf_name))
            .collect::<HashSet<_>>();
        if let Some(engine::AuthProvider::Authorizer(AuthorizerProvider { ref name })) = registry.auth.provider {
            required_udfs.insert((UdfKind::Authorizer, name.clone()));
        }

        ParseResult {
            global_cache_rules: self.global_cache_rules,
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

pub trait Visitor<'a> {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a ServiceDocument) {}
    fn exit_document(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a ServiceDocument) {}

    fn enter_schema(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a Positioned<SchemaDefinition>) {}
    fn exit_schema(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a Positioned<SchemaDefinition>) {}

    fn enter_type_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn exit_type_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn enter_object_definition(&mut self, _ctx: &mut VisitorContext<'a>, _object_definition: &'a ObjectType) {}
    fn exit_object_definition(&mut self, _ctx: &mut VisitorContext<'a>, _object_definition: &'a ObjectType) {}

    fn enter_scalar_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }
    fn exit_scalar_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _type_definition: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn enter_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Positioned<ConstDirective>) {}
    fn exit_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Positioned<ConstDirective>) {}

    fn enter_field(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
    }
    fn exit_field(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _field: &'a Positioned<FieldDefinition>,
        _parent_type: &'a Positioned<TypeDefinition>,
    ) {
    }

    fn enter_input_value_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _value: &'a Positioned<InputValueDefinition>,
    ) {
    }
    fn exit_input_value_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _value: &'a Positioned<InputValueDefinition>,
    ) {
    }

    fn enter_argument(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        _value: &'a Positioned<ConstValue>,
    ) {
    }
    fn exit_argument(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        _value: &'a Positioned<ConstValue>,
    ) {
    }
}

/// Empty Value
pub struct VisitorNil;

impl VisitorNil {
    pub(crate) const fn with<V>(self, visitor: V) -> VisitorCons<V, Self> {
        VisitorCons(visitor, self)
    }
}

/// Concat rule
pub struct VisitorCons<A, B>(A, B);

impl<A, B> VisitorCons<A, B> {
    #[allow(dead_code)]
    pub(crate) const fn with<V>(self, visitor: V) -> VisitorCons<V, Self> {
        VisitorCons(visitor, self)
    }
}

impl<'a> Visitor<'a> for VisitorNil {}

/// The monoid implementation for Visitor
impl<'a, A, B> Visitor<'a> for VisitorCons<A, B>
where
    A: Visitor<'a> + 'a,
    B: Visitor<'a> + 'a,
{
    fn enter_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        self.0.enter_schema(ctx, doc);
        self.1.enter_schema(ctx, doc);
    }

    fn exit_schema(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Positioned<SchemaDefinition>) {
        self.0.exit_schema(ctx, doc);
        self.1.exit_schema(ctx, doc);
    }

    fn enter_scalar_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a Positioned<TypeDefinition>,
    ) {
        self.0.enter_scalar_definition(ctx, type_definition);
        self.1.enter_scalar_definition(ctx, type_definition);
    }

    fn exit_scalar_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a Positioned<TypeDefinition>,
    ) {
        self.0.exit_scalar_definition(ctx, type_definition);
        self.1.exit_scalar_definition(ctx, type_definition);
    }

    fn enter_document(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a ServiceDocument) {
        self.0.enter_document(ctx, doc);
        self.1.enter_document(ctx, doc);
    }

    fn exit_document(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a ServiceDocument) {
        self.0.exit_document(ctx, doc);
        self.1.exit_document(ctx, doc);
    }

    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Positioned<ConstDirective>) {
        self.0.enter_directive(ctx, directive);
        self.1.enter_directive(ctx, directive);
    }

    fn exit_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Positioned<ConstDirective>) {
        self.0.exit_directive(ctx, directive);
        self.1.exit_directive(ctx, directive);
    }

    fn enter_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        self.0.enter_type_definition(ctx, type_definition);
        self.1.enter_type_definition(ctx, type_definition);
    }

    fn exit_type_definition(&mut self, ctx: &mut VisitorContext<'a>, type_definition: &'a Positioned<TypeDefinition>) {
        self.0.exit_type_definition(ctx, type_definition);
        self.1.exit_type_definition(ctx, type_definition);
    }

    fn enter_object_definition(&mut self, ctx: &mut VisitorContext<'a>, object_definition: &'a ObjectType) {
        self.0.enter_object_definition(ctx, object_definition);
        self.1.enter_object_definition(ctx, object_definition);
    }
    fn exit_object_definition(&mut self, ctx: &mut VisitorContext<'a>, object_definition: &'a ObjectType) {
        self.0.exit_object_definition(ctx, object_definition);
        self.1.exit_object_definition(ctx, object_definition);
    }

    fn enter_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        parent_type: &'a Positioned<TypeDefinition>,
    ) {
        self.0.enter_field(ctx, field, parent_type);
        self.1.enter_field(ctx, field, parent_type);
    }
    fn exit_field(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        field: &'a Positioned<FieldDefinition>,
        parent_type: &'a Positioned<TypeDefinition>,
    ) {
        self.0.exit_field(ctx, field, parent_type);
        self.1.exit_field(ctx, field, parent_type);
    }

    fn enter_input_value_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        value: &'a Positioned<InputValueDefinition>,
    ) {
        self.0.enter_input_value_definition(ctx, value);
        self.1.enter_input_value_definition(ctx, value);
    }
    fn exit_input_value_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        value: &'a Positioned<InputValueDefinition>,
    ) {
        self.0.exit_input_value_definition(ctx, value);
        self.1.exit_input_value_definition(ctx, value);
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        name: &'a Positioned<Name>,
        value: &'a Positioned<ConstValue>,
    ) {
        self.0.enter_argument(ctx, name, value);
        self.1.enter_argument(ctx, name, value);
    }
    fn exit_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        name: &'a Positioned<Name>,
        value: &'a Positioned<ConstValue>,
    ) {
        self.0.exit_argument(ctx, name, value);
        self.1.exit_argument(ctx, name, value);
    }
}

pub fn visit<'a, V: Visitor<'a>>(v: &mut V, ctx: &mut VisitorContext<'a>, doc: &'a ServiceDocument) {
    v.enter_document(ctx, doc);

    for operation in &doc.definitions {
        visit_type_system_definition(v, ctx, operation);
    }

    v.exit_document(ctx, doc);
}

fn visit_type_system_definition<'a, V: Visitor<'a>>(
    v: &mut V,
    ctx: &mut VisitorContext<'a>,
    operation: &'a TypeSystemDefinition,
) {
    #[allow(clippy::single_match)]
    match operation {
        TypeSystemDefinition::Type(ty) => {
            v.enter_type_definition(ctx, ty);
            // Inside Type Definition we should visit_field
            match &ty.node.kind {
                TypeKind::Object(object) => {
                    ctx.with_definition_type(Some(ty), |ctx| visit_directives(v, ctx, &ty.node.directives));

                    v.enter_object_definition(ctx, object);
                    for field in &object.fields {
                        visit_field(v, ctx, field, ty);
                    }
                    v.exit_object_definition(ctx, object);
                }
                TypeKind::Scalar => {
                    v.enter_scalar_definition(ctx, ty);
                    visit_directives(v, ctx, &ty.node.directives);
                    v.exit_scalar_definition(ctx, ty);
                }
                _ => {}
            };
            v.exit_type_definition(ctx, ty);
        }
        TypeSystemDefinition::Schema(schema) => {
            v.enter_schema(ctx, schema);
            visit_directives(v, ctx, &schema.node.directives);
            v.exit_schema(ctx, schema);
        }
        _ => {}
    };
}

fn visit_field<'a, V: Visitor<'a>>(
    v: &mut V,
    ctx: &mut VisitorContext<'a>,
    field: &'a Positioned<FieldDefinition>,
    parent_type: &'a Positioned<TypeDefinition>,
) {
    v.enter_field(ctx, field, parent_type);

    for value in &field.node.arguments {
        v.enter_input_value_definition(ctx, value);
        ctx.with_type(Some(&field.node.ty), |ctx| {
            visit_directives(v, ctx, &value.node.directives);
        });
        v.exit_input_value_definition(ctx, value);
    }

    visit_directives(v, ctx, &field.node.directives);
    v.exit_field(ctx, field, parent_type);
}

fn visit_directives<'a, V: Visitor<'a>>(
    v: &mut V,
    ctx: &mut VisitorContext<'a>,
    directives: &'a [Positioned<ConstDirective>],
) {
    for d in directives {
        v.enter_directive(ctx, d);

        // TODO: Should check than directive is inside schema defined Directives.
        for (name, value) in &d.node.arguments {
            v.enter_argument(ctx, name, value);
            v.exit_argument(ctx, name, value);
        }

        v.exit_directive(ctx, d);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RuleError {
    pub(crate) locations: Vec<Pos>,
    pub(crate) message: String,
}

impl RuleError {
    pub(crate) fn new(locations: Vec<Pos>, msg: impl Into<String>) -> Self {
        Self {
            locations,
            message: msg.into(),
        }
    }
}

impl Display for RuleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (idx, loc) in self.locations.iter().enumerate() {
            if idx == 0 {
                write!(f, "[")?;
            } else {
                write!(f, ", ")?;
            }

            write!(f, "{}:{}", loc.line, loc.column)?;

            if idx == self.locations.len() - 1 {
                write!(f, "] ")?;
            }
        }

        write!(f, "{}", self.message)?;
        Ok(())
    }
}
