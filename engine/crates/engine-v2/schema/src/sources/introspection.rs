use std::ops::{Deref, DerefMut};

use crate::{
    builder::SchemaBuilder, DataType, Definition, EnumId, EnumValue, Field, FieldId, FieldResolver, InputValue,
    InputValueId, ObjectField, ObjectId, ResolverId, ScalarId, Schema, SchemaWalker, StringId, Type, TypeId, Value,
    Wrapping,
};
use strum::EnumCount;

#[derive(Debug, PartialEq, Eq)]
pub struct Resolver;

pub type ResolverWalker<'a> = SchemaWalker<'a, &'a Resolver>;

impl<'a> ResolverWalker<'a> {
    pub fn metadata(&self) -> &'a Metadata {
        self.schema
            .data_sources
            .introspection
            .metadata
            .as_ref()
            .expect("Schema wasn't properly finalized with Introspection.")
    }
}

#[derive(Default)]
pub struct DataSource {
    // Ugly until we have some from of SchemaBuilder
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntrospectionField {
    Type,
    Schema,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::EnumCount)]
pub enum __Schema {
    Description,
    Types,
    QueryType,
    MutationType,
    SubscriptionType,
    Directives,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumCount)]
pub enum __Type {
    Kind,
    Name,
    Description,
    Fields,
    Interfaces,
    PossibleTypes,
    EnumValues,
    InputFields,
    OfType,
    SpecifiedByURL,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumCount)]
pub enum __EnumValue {
    Name,
    Description,
    IsDeprecated,
    DeprecationReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumCount)]
pub enum __InputValue {
    Name,
    Description,
    Type,
    DefaultValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumCount)]
pub enum __Field {
    Name,
    Description,
    Args,
    Type,
    IsDeprecated,
    DeprecationReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, strum_macros::EnumCount)]
pub enum __Directive {
    Name,
    Description,
    Locations,
    Args,
    IsRepeatable,
}

pub struct Metadata {
    pub meta_fields: [FieldId; 2],
    pub type_kind: TypeKind,
    pub directive_location: DirectiveLocation,
    pub __schema: IntrospectionObject<__Schema, { __Schema::COUNT }>,
    pub __type: IntrospectionObject<__Type, { __Type::COUNT }>,
    pub __enum_value: IntrospectionObject<__EnumValue, { __EnumValue::COUNT }>,
    pub __input_value: IntrospectionObject<__InputValue, { __InputValue::COUNT }>,
    pub __field: IntrospectionObject<__Field, { __Field::COUNT }>,
    pub __directive: IntrospectionObject<__Directive, { __Directive::COUNT }>,
}

pub struct IntrospectionObject<E, const N: usize> {
    pub id: ObjectId,
    pub fields: [(FieldId, E); N],
}

// Used post query validation.
impl<E: Copy, const N: usize> std::ops::Index<FieldId> for IntrospectionObject<E, N> {
    type Output = E;

    fn index(&self, index: FieldId) -> &Self::Output {
        self.fields
            .iter()
            .find_map(|(id, value)| if *id == index { Some(value) } else { None })
            .expect("Unexpected field id")
    }
}

impl Metadata {
    pub fn root_field(&self, id: FieldId) -> IntrospectionField {
        if id == self.meta_fields[0] {
            IntrospectionField::Type
        } else if id == self.meta_fields[1] {
            IntrospectionField::Schema
        } else {
            unreachable!("Unexpected field id")
        }
    }
}

pub struct TypeKind {
    pub scalar: StringId,
    pub object: StringId,
    pub interface: StringId,
    pub union: StringId,
    pub r#enum: StringId,
    pub input_object: StringId,
    pub list: StringId,
    pub non_null: StringId,
}

pub struct DirectiveLocation {
    pub query: StringId,
    pub mutation: StringId,
    pub subscription: StringId,
    pub field: StringId,
    pub fragment_definition: StringId,
    pub fragment_spread: StringId,
    pub inline_fragment: StringId,
    pub variable_definition: StringId,
    pub schema: StringId,
    pub scalar: StringId,
    pub object: StringId,
    pub field_definition: StringId,
    pub argument_definition: StringId,
    pub interface: StringId,
    pub union: StringId,
    pub r#enum: StringId,
    pub enum_value: StringId,
    pub input_object: StringId,
    pub input_field_definition: StringId,
}

pub(crate) struct IntrospectionSchemaBuilder<'a> {
    builder: &'a mut SchemaBuilder,
}

impl<'a> Deref for IntrospectionSchemaBuilder<'a> {
    type Target = Schema;
    fn deref(&self) -> &Self::Target {
        &self.builder.schema
    }
}

impl<'a> DerefMut for IntrospectionSchemaBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder.schema
    }
}

impl<'a> IntrospectionSchemaBuilder<'a> {
    pub fn insert_introspection_fields(builder: &'a mut SchemaBuilder) {
        Self { builder }.create_fields_and_insert_them()
    }

    #[allow(non_snake_case)]
    fn create_fields_and_insert_them(&mut self) {
        let nullable_string = self.find_or_create_field_type("String", DataType::String, Wrapping::nullable());
        let required_string = self.find_or_create_field_type("String", DataType::String, Wrapping::required());
        let required_boolean = self.find_or_create_field_type("Boolean", DataType::Boolean, Wrapping::required());
        let nullable_boolean = self.find_or_create_field_type("Boolean", DataType::Boolean, Wrapping::nullable());

        /*
        enum __TypeKind {
          SCALAR
          OBJECT
          INTERFACE
          UNION
          ENUM
          INPUT_OBJECT
          LIST
          NON_NULL
        }
        */
        let __type_kind = self.insert_enum(
            "__TypeKind",
            &[
                "SCALAR",
                "OBJECT",
                "INTERFACE",
                "UNION",
                "ENUM",
                "INPUT_OBJECT",
                "LIST",
                "NON_NULL",
            ],
        );
        let type_kind = TypeKind {
            scalar: self.get_or_intern("SCALAR"),
            object: self.get_or_intern("OBJECT"),
            interface: self.get_or_intern("INTERFACE"),
            union: self.get_or_intern("UNION"),
            r#enum: self.get_or_intern("ENUM"),
            input_object: self.get_or_intern("INPUT_OBJECT"),
            list: self.get_or_intern("LIST"),
            non_null: self.get_or_intern("NON_NULL"),
        };

        /*
        enum __DirectiveLocation {
          QUERY
          MUTATION
          SUBSCRIPTION
          FIELD
          FRAGMENT_DEFINITION
          FRAGMENT_SPREAD
          INLINE_FRAGMENT
          VARIABLE_DEFINITION
          SCHEMA
          SCALAR
          OBJECT
          FIELD_DEFINITION
          ARGUMENT_DEFINITION
          INTERFACE
          UNION
          ENUM
          ENUM_VALUE
          INPUT_OBJECT
          INPUT_FIELD_DEFINITION
        }
        */
        let __directive_location = self.insert_enum(
            "__DirectiveLocation",
            &[
                "QUERY",
                "MUTATION",
                "SUBSCRIPTION",
                "FIELD",
                "FRAGMENT_DEFINITION",
                "FRAGMENT_SPREAD",
                "INLINE_FRAGMENT",
                "VARIABLE_DEFINITION",
                "SCHEMA",
                "SCALAR",
                "OBJECT",
                "FIELD_DEFINITION",
                "ARGUMENT_DEFINITION",
                "INTERFACE",
                "UNION",
                "ENUM",
                "ENUM_VALUE",
                "INPUT_OBJECT",
                "INPUT_FIELD_DEFINITION",
            ],
        );
        let directive_location = DirectiveLocation {
            query: self.get_or_intern("QUERY"),
            mutation: self.get_or_intern("MUTATION"),
            subscription: self.get_or_intern("SUBSCRIPTION"),
            field: self.get_or_intern("FIELD"),
            fragment_definition: self.get_or_intern("FRAGMENT_DEFINITION"),
            fragment_spread: self.get_or_intern("FRAGMENT_SPREAD"),
            inline_fragment: self.get_or_intern("INLINE_FRAGMENT"),
            variable_definition: self.get_or_intern("VARIABLE_DEFINITION"),
            schema: self.get_or_intern("SCHEMA"),
            scalar: self.get_or_intern("SCALAR"),
            object: self.get_or_intern("OBJECT"),
            field_definition: self.get_or_intern("FIELD_DEFINITION"),
            argument_definition: self.get_or_intern("ARGUMENT_DEFINITION"),
            interface: self.get_or_intern("INTERFACE"),
            union: self.get_or_intern("UNION"),
            r#enum: self.get_or_intern("ENUM"),
            enum_value: self.get_or_intern("ENUM_VALUE"),
            input_object: self.get_or_intern("INPUT_OBJECT"),
            input_field_definition: self.get_or_intern("INPUT_FIELD_DEFINITION"),
        };

        /*
        type __EnumValue {
          name: String!
          description: String
          isDeprecated: Boolean!
          deprecationReason: String
        }
        */
        let __enum_value = self.insert_object(
            "__EnumValue",
            vec![
                ("name", required_string, __EnumValue::Name),
                ("description", nullable_string, __EnumValue::Description),
                ("isDeprecated", required_boolean, __EnumValue::IsDeprecated),
                ("deprecationReason", nullable_string, __EnumValue::DeprecationReason),
            ],
        );

        /*
        type __InputValue {
          name: String!
          description: String
          type: __Type!
          defaultValue: String
        }
        */
        let mut __input_value = self.insert_object(
            "__InputValue",
            vec![
                ("name", required_string, __InputValue::Name),
                ("description", nullable_string, __InputValue::Description),
                // type added later
                ("defaultValue", nullable_string, __InputValue::DefaultValue),
            ],
        );
        let args = self.insert_field_type(__input_value.id, Wrapping::required().wrapped_by_required_list());

        /*
        type __Field {
          name: String!
          description: String
          args: [__InputValue!]!
          type: __Type!
          isDeprecated: Boolean!
          deprecationReason: String
        }
        */
        let mut __field = self.insert_object(
            "__Field",
            vec![
                ("name", required_string, __Field::Name),
                ("description", nullable_string, __Field::Description),
                ("args", args, __Field::Args),
                // type added later
                ("isDeprecated", required_boolean, __Field::IsDeprecated),
                ("deprecationReason", nullable_string, __Field::DeprecationReason),
            ],
        );

        /*
        type __Directive {
          name: String!
          description: String
          locations: [__DirectiveLocation!]!
          args: [__InputValue!]!
          isRepeatable: Boolean!
        }
        */
        let __directive = {
            let locations =
                self.insert_field_type(__directive_location, Wrapping::required().wrapped_by_required_list());
            self.insert_object(
                "__Directive",
                vec![
                    ("name", required_string, __Directive::Name),
                    ("description", nullable_string, __Directive::Description),
                    ("locations", locations, __Directive::Locations),
                    ("args", args, __Directive::Args),
                    ("isRepeatable", required_boolean, __Directive::IsRepeatable),
                ],
            )
        };

        /*
        type __Type {
          kind: __TypeKind!
          name: String
          description: String
          fields(includeDeprecated: Boolean = false): [__Field!]
          interfaces: [__Type!]
          possibleTypes: [__Type!]
          enumValues(includeDeprecated: Boolean = false): [__EnumValue!]
          inputFields: [__InputValue!]
          ofType: __Type
          specifiedByURL: String
        }
        */
        let mut __type = {
            let kind = self.insert_field_type(__type_kind, Wrapping::required());
            let input_fields =
                self.insert_field_type(__input_value.id, Wrapping::required().wrapped_by_nullable_list());
            let mut __type = self.insert_object(
                "__Type",
                vec![
                    ("kind", kind, __Type::Kind),
                    ("name", nullable_string, __Type::Name),
                    ("description", nullable_string, __Type::Description),
                    ("inputFields", input_fields, __Type::InputFields),
                    ("specifiedByURL", nullable_string, __Type::SpecifiedByURL),
                    // other fields added later
                ],
            );
            {
                let nullable__field_list =
                    self.insert_field_type(__field.id, Wrapping::required().wrapped_by_nullable_list());
                let field_id = self.insert_object_field(__type.id, "fields", nullable__field_list);
                __type.fields.push((field_id, __Type::Fields));
                let input_value_id =
                    self.insert_input_value("includeDeprecated", nullable_boolean, Some(Value::Boolean(false)));
                self[field_id].arguments.push(input_value_id);
            }
            {
                let nullable__enum_value_list =
                    self.insert_field_type(__enum_value.id, Wrapping::required().wrapped_by_nullable_list());
                let field_id = self.insert_object_field(__type.id, "enumValues", nullable__enum_value_list);
                __type.fields.push((field_id, __Type::EnumValues));
                let input_value_id =
                    self.insert_input_value("includeDeprecated", nullable_boolean, Some(Value::Boolean(false)));
                self[field_id].arguments.push(input_value_id);
            }
            __type
        };

        let required__type = self.insert_field_type(__type.id, Wrapping::required());
        let nullable__type = self.insert_field_type(__type.id, Wrapping::nullable());
        let required__type_list = self.insert_field_type(__type.id, Wrapping::required().wrapped_by_required_list());
        let nullable__type_list = self.insert_field_type(__type.id, Wrapping::required().wrapped_by_nullable_list());

        __input_value.fields.push((
            self.insert_object_field(__input_value.id, "type", required__type),
            __InputValue::Type,
        ));
        __field.fields.push((
            self.insert_object_field(__field.id, "type", required__type),
            __Field::Type,
        ));
        __type.fields.push((
            self.insert_object_field(__type.id, "ofType", nullable__type),
            __Type::OfType,
        ));
        __type.fields.push((
            self.insert_object_field(__type.id, "possibleTypes", nullable__type_list),
            __Type::PossibleTypes,
        ));
        __type.fields.push((
            self.insert_object_field(__type.id, "interfaces", nullable__type_list),
            __Type::Interfaces,
        ));

        /*
        type __Schema {
          description: String
          types: [__Type!]!
          queryType: __Type!
          mutationType: __Type
          subscriptionType: __Type
          directives: [__Directive!]!
        }
        */
        let required__directive_list =
            self.insert_field_type(__directive.id, Wrapping::required().wrapped_by_required_list());
        let __schema = self.insert_object(
            "__Schema",
            vec![
                ("description", nullable_string, __Schema::Description),
                ("types", required__type_list, __Schema::Types),
                ("queryType", required__type, __Schema::QueryType),
                ("mutationType", nullable__type, __Schema::MutationType),
                ("subscriptionType", nullable__type, __Schema::SubscriptionType),
                ("directives", required__directive_list, __Schema::Directives),
            ],
        );

        let resolver_id = ResolverId::from(self.resolvers.len());
        self.resolvers.push(crate::Resolver::Introspection(Resolver));

        /*
        __schema: __Schema!
        */
        let field_type_id = self.insert_field_type(__schema.id, Wrapping::required());
        let __schema_field_id = self.insert_object_field(self.root_operation_types.query, "__schema", field_type_id);
        self[__schema_field_id].resolvers.push(FieldResolver {
            resolver_id,
            field_requires: Default::default(),
        });

        /*
        __type(name: String!): __Type
        */
        let field_type_id = self.insert_field_type(__type.id, Wrapping::nullable());
        let __type_field_id = self.insert_object_field(self.root_operation_types.query, "__type", field_type_id);
        self[__type_field_id].resolvers.push(FieldResolver {
            resolver_id,
            field_requires: Default::default(),
        });
        let input_value_id = self.insert_input_value("name", required_string, None);
        self[__type_field_id].arguments.push(input_value_id);

        // DataSource
        self.data_sources.introspection.metadata = Some(Metadata {
            meta_fields: [__type_field_id, __schema_field_id],
            type_kind,
            directive_location,
            __schema: __schema.into(),
            __type: __type.into(),
            __enum_value: __enum_value.into(),
            __input_value: __input_value.into(),
            __field: __field.into(),
            __directive: __directive.into(),
        });
    }

    fn insert_enum(&mut self, name: &str, values: &[&str]) -> EnumId {
        let name = self.get_or_intern(name);
        let values = values
            .iter()
            .map(|value| {
                let value = self.get_or_intern(value);
                EnumValue {
                    name: value,
                    composed_directives: vec![],
                    description: None,
                    is_deprecated: false,
                    deprecation_reason: None,
                }
            })
            .collect();

        self.enums.push(crate::Enum {
            name,
            description: None,
            values,
            composed_directives: vec![],
        });
        let enum_id = EnumId::from(self.enums.len() - 1);
        self.definitions.push(Definition::Enum(enum_id));
        enum_id
    }

    fn new_object(&mut self, name: &str) -> ObjectId {
        let name = self.get_or_intern(name);
        self.objects.push(crate::Object {
            name,
            description: None,
            interfaces: vec![],
            composed_directives: vec![],
            cache_config: None,
        });
        ObjectId::from(self.objects.len() - 1)
    }

    fn insert_object_field(&mut self, object_id: ObjectId, name: &str, field_type_id: TypeId) -> FieldId {
        let name = self.get_or_intern(name);
        self.fields.push(Field {
            name,
            type_id: field_type_id,
            composed_directives: vec![],
            resolvers: vec![],
            provides: vec![],
            arguments: vec![],
            description: None,
            is_deprecated: false,
            deprecation_reason: None,
            cache_config: None,
        });
        let field_id = FieldId::from(self.fields.len() - 1);
        self.object_fields.push(ObjectField { object_id, field_id });
        field_id
    }

    fn insert_object<E>(&mut self, name: &str, fields: Vec<(&str, TypeId, E)>) -> IncompleteIntrospectionObject<E> {
        let id = self.new_object(name);
        self.definitions.push(Definition::from(id));
        IncompleteIntrospectionObject {
            id,
            fields: fields
                .into_iter()
                .map(|(name, field_type_id, field_enum)| {
                    (self.insert_object_field(id, name, field_type_id), field_enum)
                })
                .collect(),
        }
    }

    fn insert_field_type(&mut self, kind: impl Into<Definition>, wrapping: Wrapping) -> TypeId {
        self.types.push(Type {
            inner: kind.into(),
            wrapping,
        });
        TypeId::from(self.types.len() - 1)
    }

    fn insert_input_value(&mut self, name: &str, type_id: TypeId, default_value: Option<Value>) -> InputValueId {
        let name = self.get_or_intern(name);
        self.input_values.push(InputValue {
            name,
            description: None,
            default_value,
            type_id,
        });
        InputValueId::from(self.input_values.len() - 1)
    }

    fn find_or_create_field_type(
        &mut self,
        scalar_name: &str,
        scalar_type: DataType,
        expected_wrapping: Wrapping,
    ) -> TypeId {
        let scalar_id = match self
            .scalars
            .iter()
            .enumerate()
            .find(|(_, scalar)| self.builder.strings[scalar.name] == scalar_name)
            .map(|(id, _)| ScalarId::from(id))
        {
            Some(id) => id,
            None => {
                let name = self.builder.strings.get_or_insert(scalar_name);
                self.scalars.push(crate::Scalar {
                    name,
                    data_type: scalar_type,
                    description: None,
                    specified_by_url: None,
                    composed_directives: vec![],
                });
                ScalarId::from(self.scalars.len() - 1)
            }
        };
        let expected_kind = Definition::from(scalar_id);
        match self
            .types
            .iter()
            .enumerate()
            .find(|(_, Type { inner: kind, wrapping })| kind == &expected_kind && wrapping == &expected_wrapping)
        {
            Some((id, _)) => TypeId::from(id),
            None => self.insert_field_type(expected_kind, expected_wrapping),
        }
    }

    fn get_or_intern(&mut self, value: &str) -> StringId {
        self.builder.strings.get_or_insert(value)
    }
}

struct IncompleteIntrospectionObject<E> {
    id: ObjectId,
    fields: Vec<(FieldId, E)>,
}

impl<E: std::fmt::Debug, const N: usize> From<IncompleteIntrospectionObject<E>> for IntrospectionObject<E, N> {
    fn from(value: IncompleteIntrospectionObject<E>) -> Self {
        IntrospectionObject {
            id: value.id,
            fields: value.fields.try_into().unwrap(),
        }
    }
}
