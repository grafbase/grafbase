//! Functions on the registry.
//!
//! These should be vetted and deleted if they're not needed

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use indexmap::{IndexMap, IndexSet};
use registry_v2::{
    resolvers::{introspection::IntrospectionResolver, Resolver},
    DirectiveLocation, MongoDBConfiguration, OperationType, ScalarParser,
};

use crate::{
    EnumType, InputObjectType, InputValueType, InterfaceType, MetaDirective, MetaField, MetaFieldType, MetaInputValue,
    MetaType, ObjectType, Registry, ScalarType, UnionType,
};

impl Registry {
    pub fn new() -> Registry {
        let type_query = "Query".to_string();
        let mut registry = Registry {
            query_type: type_query.clone(),
            ..Registry::default()
        };
        registry
            .types
            .insert(type_query.clone(), ObjectType::new(type_query, []).into());
        registry
    }

    /// Fill the `Registry` with sample data.
    ///
    /// This can be useful for testing purposes.
    pub fn with_sample_data(mut self) -> Self {
        let fields = self.query_root_mut().fields_mut().unwrap();

        fields.insert(
            "scalar".to_owned(),
            MetaField {
                name: "scalar".to_owned(),
                description: Some("test scalar".to_owned()),
                ty: "MyScalar".into(),
                ..Default::default()
            },
        );

        self.types.insert(
            "MyScalar".to_owned(),
            MetaType::Scalar(ScalarType {
                name: "MyScalar".to_owned(),
                description: Some("test scalar".to_owned()),
                is_valid: None,
                specified_by_url: None,
                parser: ScalarParser::default(),
            }),
        );

        self
    }

    pub fn query_root(&self) -> &MetaType {
        self.types.get(&self.query_type).unwrap()
    }

    pub fn query_root_mut(&mut self) -> &mut MetaType {
        self.types.get_mut(&self.query_type).unwrap()
    }

    pub fn mutation_root(&self) -> &MetaType {
        self.types.get(self.mutation_type.as_deref().unwrap()).unwrap()
    }

    pub fn mutation_root_mut(&mut self) -> &mut MetaType {
        self.types.get_mut(self.mutation_type.as_deref().unwrap()).unwrap()
    }
}

impl Registry {
    pub fn insert_type(&mut self, ty: impl Into<MetaType>) {
        let ty = ty.into();
        self.types.insert(ty.name().to_string(), ty);
    }

    pub fn create_mongo_config<F>(&mut self, f: F, name: &str)
    where
        F: FnOnce(&mut Registry) -> MongoDBConfiguration,
    {
        if self.mongodb_configurations.contains_key(name) {
            panic!("MongoDB directive with `{name}` already exists.");
        }

        let config = f(self);
        self.mongodb_configurations.insert(name.to_string(), config);
    }

    pub fn create_type<F: FnOnce(&mut Registry) -> MetaType>(&mut self, f: F, name: &str, rust_typename: &str) {
        match self.types.get(name) {
            Some(ty) => {
                if let Some(prev_typename) = ty.rust_typename() {
                    if prev_typename.ne("__fake_type__") && prev_typename.ne(rust_typename) {
                        panic!("`{prev_typename}` and `{rust_typename}` have the same GraphQL name `{name}`",);
                    }
                }
            }
            None => {
                // Inserting a fake type before calling the function allows recursive types to exist.
                self.types.insert(
                    name.to_string(),
                    ObjectType {
                        rust_typename: "__fake_type__".to_string(),
                        ..ObjectType::new(String::new(), [])
                    }
                    .into(),
                );
                let ty = f(self);
                *self.types.get_mut(name).unwrap() = ty;
            }
        }
    }

    pub fn add_directive(&mut self, directive: MetaDirective) {
        self.directives.insert(directive.name.to_string(), directive);
    }

    pub fn add_implements(&mut self, ty: &str, interface: &str) {
        self.implements
            .entry(ty.to_string())
            .and_modify(|interfaces| {
                interfaces.insert(interface.to_string());
            })
            .or_insert({
                let mut interfaces = HashSet::new();
                interfaces.insert(interface.to_string());
                interfaces
            });
    }

    pub fn add_builtins_to_registry(&mut self) {
        self.add_directive(MetaDirective {
            name: "include".to_string(),
            description: Some(
                "Directs the executor to include this field or fragment only when the `if` argument is true."
                    .to_string(),
            ),
            locations: vec![
                DirectiveLocation::Field,
                DirectiveLocation::FragmentSpread,
                DirectiveLocation::InlineFragment,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "if".to_string(),
                    MetaInputValue::new("if".to_string(), "Boolean!").with_description("Included when true."),
                );
                args
            },
            is_repeatable: false,
        });

        self.add_directive(MetaDirective {
            name: "skip".to_string(),
            description: Some(
                "Directs the executor to skip this field or fragment when the `if` argument is true.".to_string(),
            ),
            locations: vec![
                DirectiveLocation::Field,
                DirectiveLocation::FragmentSpread,
                DirectiveLocation::InlineFragment,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "if".to_string(),
                    MetaInputValue::new("if", "Boolean!").with_description("Skipped when true."),
                );
                args
            },
            is_repeatable: false,
        });

        self.add_directive(MetaDirective {
            name: "oneOf".to_string(),
            description: Some("Indicates that an input object is a oneOf input object".to_string()),
            locations: vec![DirectiveLocation::InputObject],
            args: IndexMap::new(),
            is_repeatable: false,
        });

        self.add_directive(MetaDirective {
            name: "defer".to_string(),
            description: Some("De-prioritizes a fragment, causing the fragment to be omitted in the initial response and delivered as a subsequent response afterward.".to_string()),
            locations: vec![
                DirectiveLocation::InlineFragment,
                DirectiveLocation::FragmentSpread
            ],
            args: [
                MetaInputValue::new("if", "Boolean!")
                    .with_description("When true fragment may be deferred")
                    .with_default(engine_value::ConstValue::Boolean(true)),
                MetaInputValue::new("label", "String")
                    .with_description("This label should be used by GraphQL clients to identify the data from patch responses and associate it with the correct fragment.")
            ]
                .into_iter()
                .map(|directive| (directive.name.clone(), directive))
                .collect(),
            is_repeatable: false,
        });

        // register scalars
        for builtin in ["Boolean", "Int", "Float", "String", "ID"] {
            self.types.insert(
                builtin.to_string(),
                MetaType::Scalar(ScalarType {
                    name: builtin.to_string(),
                    description: None,
                    is_valid: None,
                    specified_by_url: None,
                    parser: ScalarParser::BestEffort,
                }),
            );
        }
    }

    pub fn has_entities(&self) -> bool {
        !self.federation_entities.is_empty()
    }

    /// Each type annotated with @key should be added to the _Entity union.
    /// If no types are annotated with the key directive, then the _Entity union
    /// and Query._entities field should be removed from the schema.
    ///
    /// [Reference](https://www.apollographql.com/docs/federation/federation-spec/#resolve-requests-for-entities).
    fn create_entity_type_and_root_field(&mut self) {
        let possible_types: IndexSet<_> = self.federation_entities.keys().cloned().collect();

        if !possible_types.is_empty() {
            self.types.insert(
                "_Entity".to_string(),
                UnionType {
                    name: "_Entity".to_string(),
                    description: None,
                    possible_types,
                    rust_typename: "engine::federation::Entity".to_string(),
                    discriminators: None,
                }
                .into(),
            );

            let query_root = self.types.get_mut(&self.query_type).unwrap();
            if let MetaType::Object(object) = query_root {
                object.fields.insert(
                    "_service".to_string(),
                    MetaField {
                        name: "_service".to_string(),
                        ty: "_Service!".into(),
                        resolver: Resolver::Introspection(IntrospectionResolver::FederationServiceField),
                        ..Default::default()
                    },
                );

                object.fields.insert(
                    "_entities".to_string(),
                    MetaField {
                        name: "_entities".to_string(),
                        args: {
                            let mut args = IndexMap::new();
                            args.insert(
                                "representations".to_string(),
                                MetaInputValue::new("representations", "[_Any!]!"),
                            );
                            args
                        },
                        ty: "[_Entity]!".into(),
                        resolver: Resolver::FederationEntitiesResolver,
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn create_federation_types(&mut self) {
        self.types.insert(
            "_Any".into(),
            ScalarType {
                name: "_Any".into(),
                description: Some(indoc::indoc! { r#"
                    Any scalar (For [Apollo Federation](https://www.apollographql.com/docs/apollo-server/federation/introduction))

                    The `Any` scalar is used to pass representations of entities from external services into the root `_entities` field for execution.
                "#}.to_string()),
                is_valid: None,
                specified_by_url: None,
                parser: ScalarParser::BestEffort,
            }
            .into(),
        );

        self.types.insert(
            "_Service".to_string(),
            ObjectType {
                rust_typename: "engine::federation::Service".to_string(),
                ..ObjectType::new(
                    "_Service",
                    [MetaField {
                        name: "sdl".to_string(),
                        ty: "String".into(),
                        ..Default::default()
                    }],
                )
            }
            .into(),
        );

        self.create_entity_type_and_root_field();
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = HashSet::new();

        for d in self.directives.values() {
            names.insert(d.name.to_string());
            names.extend(d.args.values().map(|arg| arg.name.to_string()));
        }

        for ty in self.types.values() {
            match ty {
                MetaType::Scalar(_) | MetaType::Union(_) => {
                    names.insert(ty.name().to_string());
                }
                MetaType::Object(ObjectType { name, fields, .. })
                | MetaType::Interface(InterfaceType { name, fields, .. }) => {
                    names.insert(name.clone());
                    names.extend(fields.values().flat_map(|field| {
                        std::iter::once(field.name.clone()).chain(field.args.values().map(|arg| arg.name.to_string()))
                    }));
                }
                MetaType::Enum(EnumType { name, enum_values, .. }) => {
                    names.insert(name.clone());
                    names.extend(enum_values.values().map(|value| value.name.to_string()));
                }
                MetaType::InputObject(InputObjectType { name, input_fields, .. }) => {
                    names.insert(name.clone());
                    names.extend(input_fields.values().map(|field| field.name.to_string()));
                }
            }
        }

        names.into_iter().collect()
    }

    pub fn set_description(&mut self, name: &str, desc: &'static str) {
        match self.types.get_mut(name) {
            Some(MetaType::Scalar(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Object(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Interface(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Union(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::Enum(inner)) => inner.description = Some(desc.to_string()),
            Some(MetaType::InputObject(inner)) => {
                inner.description = Some(desc.to_string());
            }
            None => {}
        }
    }

    pub fn remove_unused_types(&mut self) {
        self.populate_possible_types();

        let mut used_types = BTreeSet::new();
        let mut unused_types = HashSet::new();

        fn traverse_field<'a>(
            types: &'a BTreeMap<String, MetaType>,
            implements: &'a HashMap<String, HashSet<String>>,
            used_types: &mut BTreeSet<&'a str>,
            field: &'a MetaField,
        ) {
            traverse_type(types, implements, used_types, named_type_from_type_str(&field.ty.0));
            for arg in field.args.values() {
                traverse_input_value(types, implements, used_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            types: &'a BTreeMap<String, MetaType>,
            implements: &'a HashMap<String, HashSet<String>>,
            used_types: &mut BTreeSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            traverse_type(
                types,
                implements,
                used_types,
                named_type_from_type_str(&input_value.ty.0),
            );
        }

        fn traverse_type<'a>(
            types: &'a BTreeMap<String, MetaType>,
            implements: &'a HashMap<String, HashSet<String>>,
            used_types: &mut BTreeSet<&'a str>,
            type_name: &str,
        ) {
            if used_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                used_types.insert(ty.name());
                match ty {
                    MetaType::Object(object) => {
                        for field in object.fields.values() {
                            traverse_field(types, implements, used_types, field);
                        }
                    }
                    MetaType::Interface(interface) => {
                        for field in interface.fields.values() {
                            traverse_field(types, implements, used_types, field);
                        }
                        for type_name in &interface.possible_types {
                            traverse_type(types, implements, used_types, type_name);
                        }
                    }
                    MetaType::Union(union_type) => {
                        for type_name in &union_type.possible_types {
                            traverse_type(types, implements, used_types, type_name);
                        }
                    }
                    MetaType::InputObject(input_object) => {
                        for field in input_object.input_fields.values() {
                            traverse_input_value(types, implements, used_types, field);
                        }
                    }
                    _ => {}
                }
            }
        }

        for directive in self.directives.values() {
            for arg in directive.args.values() {
                traverse_input_value(&self.types, &self.implements, &mut used_types, arg);
            }
        }

        let used_interfaces: HashSet<&String> = self.implements.values().flatten().collect();
        for type_name in Some(&self.query_type)
            .into_iter()
            .chain(self.mutation_type.iter())
            .chain(self.subscription_type.iter())
            .chain(used_interfaces)
        {
            traverse_type(&self.types, &self.implements, &mut used_types, type_name);
        }

        for ty in self.federation_entities.keys() {
            traverse_type(&self.types, &self.implements, &mut used_types, ty);
        }

        for ty in self.types.values() {
            let name = ty.name();
            if !is_system_type(name) && !used_types.contains(name) {
                unused_types.insert(name.to_string());
            }
        }

        for type_name in &unused_types {
            self.types.remove(type_name);
        }
    }

    /// Populates the possible_types field of Interfaces
    fn populate_possible_types(&mut self) {
        for (possible_type, interfaces) in &self.implements {
            for interface_name in interfaces {
                if let Some(MetaType::Interface(ref mut interface)) = self.types.get_mut(interface_name) {
                    interface.possible_types.insert(possible_type.clone());
                }
            }
        }
    }

    pub fn remove_empty_types(&mut self) {
        let mut types_to_be_removed = Vec::new();
        let mut fields_to_be_removed = Vec::new();

        loop {
            for r#type in self.types.values().filter(|ty| ty.is_object()) {
                match r#type.fields() {
                    None => types_to_be_removed.push(r#type.name().to_owned()),
                    Some(fields) if fields.is_empty() => types_to_be_removed.push(r#type.name().to_owned()),
                    Some(_) => (),
                }
            }

            types_to_be_removed.sort();

            for r#type in self.types.values().filter(|ty| ty.is_object()) {
                for field in r#type.fields().into_iter().flat_map(|fields| fields.values()) {
                    if types_to_be_removed
                        .binary_search_by_key(&named_type_from_type_str(&field.ty.0), |ty| ty.as_str())
                        .is_ok()
                    {
                        fields_to_be_removed.push((r#type.name().to_owned(), field.name.clone()));
                    };
                }
            }

            if types_to_be_removed.is_empty() && fields_to_be_removed.is_empty() {
                break;
            }

            for type_name in types_to_be_removed.drain(..) {
                self.types.remove(&type_name);
            }

            for (type_name, field_name) in fields_to_be_removed.drain(..) {
                if self.mutation_type.as_ref() == Some(&type_name) {
                    self.mutation_type = None
                }

                if self.subscription_type.as_ref() == Some(&type_name) {
                    self.subscription_type = None
                }

                let Some(ty) = self.types.get_mut(&type_name) else {
                    continue;
                };

                let Some(fields) = ty.fields_mut() else { continue };

                fields.shift_remove(&field_name);
            }
        }
    }
}

fn is_system_type(name: &str) -> bool {
    if name.starts_with("__") {
        return true;
    }

    name == "Boolean" || name == "Int" || name == "Float" || name == "String" || name == "ID"
}

/// Strips the NonNull and List wrappers from a type string to get the
/// named type within.
pub(super) fn named_type_from_type_str(meta: &str) -> &str {
    let mut nested = Some(meta);

    if meta.starts_with('[') && meta.ends_with(']') {
        nested = nested.and_then(|x| x.strip_prefix('['));
        nested = nested.and_then(|x| x.strip_suffix(']'));
        return named_type_from_type_str(nested.expect("Can't fail"));
    }

    if meta.ends_with('!') {
        nested = nested.and_then(|x| x.strip_suffix('!'));
        return named_type_from_type_str(nested.expect("Can't fail"));
    }

    nested.expect("Can't fail")
}

/// A trait for types that represent type names in someway.
///
/// This is used by the lookup function on the `Registry` to provide a bit of convenience
/// and type-safety around retrieving types from the registry.
pub trait TypeReference {
    /// The name of the type
    fn name(&self) -> &str;
}

impl TypeReference for MetaFieldType {
    fn name(&self) -> &str {
        self.base_type_name()
    }
}

impl TypeReference for InputValueType {
    fn name(&self) -> &str {
        self.base_type_name()
    }
}

impl Registry {
    /// Looks up a particular type in the registry, using the default kind for the given TypeName.
    ///
    /// Will error if the type doesn't exist or is of an unexpected kind.
    pub fn lookup<'a, Name>(&'a self, name: &Name) -> Option<&'a MetaType>
    where
        Name: TypeReference,
    {
        self.lookup_by_str(name.name())
    }

    fn lookup_by_str<'a>(&'a self, name: &str) -> Option<&'a MetaType> {
        self.types.get(name)
    }

    pub fn root_type(&self, operation_type: OperationType) -> &MetaType {
        match operation_type {
            OperationType::Query => self.query_root(),
            OperationType::Mutation => self.mutation_root(),
            OperationType::Subscription => {
                // We don't do subscriptions but may as well implement anyway.
                unimplemented!()
            }
        }
    }
}
