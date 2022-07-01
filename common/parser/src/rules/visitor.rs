use dynaql::indexmap::IndexMap;
use dynaql::model::__Schema;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::{MetaField, Registry};
use dynaql::{Name, OutputType, Pos, Positioned, Schema};
use dynaql_parser::types::{
    ConstDirective, DirectiveDefinition, FieldDefinition, InputValueDefinition, ObjectType, SchemaDefinition,
    ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use dynaql_value::ConstValue;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

type TypeStackType<'a> = Vec<(Option<&'a Positioned<Type>>, Option<&'a Positioned<TypeDefinition>>)>;

/// The VisitorContext to visit every types from the Schema.
pub struct VisitorContext<'a> {
    #[allow(dead_code)]
    pub(crate) directives: HashMap<String, &'a Positioned<DirectiveDefinition>>,
    #[allow(dead_code)]
    pub(crate) types: HashMap<String, Cow<'a, Positioned<TypeDefinition>>>,
    #[allow(dead_code)]
    pub(crate) schema: Vec<&'a Positioned<SchemaDefinition>>,
    pub(crate) errors: Vec<RuleError>,
    pub(crate) type_stack: TypeStackType<'a>,
    pub(crate) queries: Vec<MetaField>,
    pub(crate) mutations: Vec<MetaField>,
    /// Relations by name
    pub(crate) relations: IndexMap<String, MetaRelation>,
    pub registry: RefCell<Registry>,
}

/// Add a fake scalar to the types HashMap if it isn't added by the schema.
fn add_fake_scalar<'a>(types: &mut HashMap<String, Cow<'a, Positioned<TypeDefinition>>>, name: &str) {
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
    pub(crate) fn new(document: &'a ServiceDocument) -> Self {
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
        }
    }

    /// Finish the Registry
    pub(crate) fn finish(self) -> Registry {
        let mut registry = self.registry.take();
        if !self.mutations.is_empty() {
            registry.mutation_type = Some("Mutation".to_string());
        }

        registry.create_type(
            &mut |registry| {
                let schema_type = __Schema::create_type_info(registry);
                dynaql::registry::MetaType::Object {
                    name: "Query".to_owned(),
                    description: None,
                    fields: {
                        let mut fields = dynaql::indexmap::IndexMap::new();
                        fields.insert(
                            "__schema".to_string(),
                            MetaField {
                                name: "__schema".to_string(),
                                description: Some("Access the current type schema of this server.".to_string()),
                                args: Default::default(),
                                ty: schema_type,
                                deprecation: Default::default(),
                                cache_control: Default::default(),
                                external: false,
                                requires: None,
                                provides: None,
                                visible: None,
                                compute_complexity: None,
                                resolve: None,
                                edges: Vec::new(),
                                relation: None,
                                transforms: None,
                            },
                        );
                        for query in &self.queries {
                            fields.insert(query.name.clone(), query.clone());
                        }
                        fields
                    },
                    cache_control: dynaql::CacheControl {
                        public: true,
                        max_age: 0usize,
                    },
                    extends: false,
                    keys: ::std::option::Option::None,
                    visible: ::std::option::Option::None,
                    is_subscription: false,
                    is_node: false,
                    rust_typename: "Query".to_owned(),
                }
            },
            "Query",
            "Query",
        );

        if !self.mutations.is_empty() {
            registry.create_type(
                &mut |_| dynaql::registry::MetaType::Object {
                    name: "Mutation".to_owned(),
                    description: None,
                    fields: {
                        let mut fields = dynaql::indexmap::IndexMap::new();
                        for mutation in &self.mutations {
                            fields.insert(mutation.name.clone(), mutation.clone());
                        }
                        fields
                    },
                    cache_control: dynaql::CacheControl {
                        public: true,
                        max_age: 0usize,
                    },
                    extends: false,
                    keys: ::std::option::Option::None,
                    visible: ::std::option::Option::None,
                    is_subscription: false,
                    is_node: false,
                    rust_typename: "Mutation".to_owned(),
                },
                "Mutation",
                "Mutation",
            );
        }

        registry.remove_unused_types();
        registry
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
}

pub trait Visitor<'a> {
    fn directives(&self) -> String {
        String::new()
    }

    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a ServiceDocument) {}
    fn exit_document(&mut self, _ctx: &mut VisitorContext<'a>, _doc: &'a ServiceDocument) {}

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

    /*
    fn enter_input_value(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _pos: Pos,
        _expected_type: &Option<MetaTypeName<'a>>,
        _value: &'a Value,
    ) {
    }
    fn exit_input_value(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _pos: Pos,
        _expected_type: &Option<MetaTypeName<'a>>,
        _value: &Value,
    ) {
    }
    */
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

impl<'a> Visitor<'a> for VisitorNil {
    fn directives(&self) -> String {
        "".to_owned()
    }
}

/// The monoid implementation for Visitor
impl<'a, A, B> Visitor<'a> for VisitorCons<A, B>
where
    A: Visitor<'a> + 'a,
    B: Visitor<'a> + 'a,
{
    fn directives(&self) -> String {
        format!("{}\n{}", self.0.directives(), self.1.directives())
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

            ctx.with_definition_type(Some(ty), |ctx| visit_directives(v, ctx, &ty.node.directives));
            visit_directives(v, ctx, &ty.node.directives);
            // Inside Type Definition we should visit_field
            match &ty.node.kind {
                TypeKind::Object(object) => {
                    v.enter_object_definition(ctx, object);
                    for field in &object.fields {
                        visit_field(v, ctx, field, ty);
                    }
                    v.exit_object_definition(ctx, object);
                }
                _ => {}
            };
            v.exit_type_definition(ctx, ty);
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

#[derive(Debug, PartialEq)]
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
