use std::ops::{Deref, DerefMut};

use crate::{
    builder::SchemaBuilder, Definition, EnumId, EnumValue, EnumValueId, FieldDefinition, FieldDefinitionId,
    FieldResolver, IdRange, InputValueDefinition, InputValueDefinitionId, ObjectId, RequiredFieldSetId, ResolverId,
    ScalarId, ScalarType, Schema, SchemaInputValue, SchemaInputValueId, SchemaWalker, StringId, Type, Wrapping,
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
    pub meta_fields: [FieldDefinitionId; 2],
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
    pub fields: [(FieldDefinitionId, E); N],
}

// Used post query validation.
impl<E: Copy, const N: usize> std::ops::Index<FieldDefinitionId> for IntrospectionObject<E, N> {
    type Output = E;

    fn index(&self, index: FieldDefinitionId) -> &Self::Output {
        self.fields
            .iter()
            .find_map(|(id, value)| if *id == index { Some(value) } else { None })
            .expect("Unexpected field id")
    }
}

impl Metadata {
    pub fn root_field(&self, id: FieldDefinitionId) -> IntrospectionField {
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
    empty_requires_id: RequiredFieldSetId,
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
    pub fn insert_introspection_fields(builder: &'a mut SchemaBuilder, empty_requires_id: RequiredFieldSetId) {
        Self {
            builder,
            empty_requires_id,
        }
        .create_fields_and_insert_them()
    }

    #[allow(non_snake_case)]
    fn create_fields_and_insert_them(&mut self) {
        let nullable_string = self.field_type("String", ScalarType::String, Wrapping::nullable());
        let required_string = self.field_type("String", ScalarType::String, Wrapping::required());
        let required_boolean = self.field_type("Boolean", ScalarType::Boolean, Wrapping::required());
        let nullable_boolean = self.field_type("Boolean", ScalarType::Boolean, Wrapping::nullable());

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
        let __enum_value = self.insert_object("__EnumValue");

        let __enum_value = self.insert_object_fields(
            __enum_value,
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
        let mut __input_value = self.insert_object("__InputValue");

        let args = Type {
            inner: __input_value.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };

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
        let mut __field = self.insert_object("__Field");

        /*
        type __Directive {
          name: String!
          description: String
          locations: [__DirectiveLocation!]!
          args: [__InputValue!]!
          isRepeatable: Boolean!
        }
        */
        let __directive = self.insert_object("__Directive");

        let locations = Type {
            inner: __directive_location.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };

        let __directive = self.insert_object_fields(
            __directive,
            vec![
                ("name", required_string, __Directive::Name),
                ("description", nullable_string, __Directive::Description),
                ("locations", locations, __Directive::Locations),
                ("args", args, __Directive::Args),
                ("isRepeatable", required_boolean, __Directive::IsRepeatable),
            ],
        );

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
        let __type = self.insert_object("__Type");

        let kind = Type {
            inner: __type_kind.into(),
            wrapping: Wrapping::required(),
        };
        let input_fields = Type {
            inner: __input_value.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };
        let nullable__field_list = Type {
            inner: __field.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };
        let nullable__enum_value_list = Type {
            inner: __enum_value.id.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };

        let required__type = Type {
            inner: __type.into(),
            wrapping: Wrapping::required(),
        };
        let nullable__type = Type {
            inner: __type.into(),
            wrapping: Wrapping::nullable(),
        };
        let required__type_list = Type {
            inner: __type.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };
        let nullable__type_list = Type {
            inner: __type.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };

        let __type = self.insert_object_fields(
            __type,
            vec![
                ("kind", kind, __Type::Kind),
                ("name", nullable_string, __Type::Name),
                ("description", nullable_string, __Type::Description),
                ("inputFields", input_fields, __Type::InputFields),
                ("specifiedByURL", nullable_string, __Type::SpecifiedByURL),
                ("fields", nullable__field_list, __Type::Fields),
                ("enumValues", nullable__enum_value_list, __Type::EnumValues),
                ("ofType", nullable__type, __Type::OfType),
                ("possibleTypes", nullable__type_list, __Type::PossibleTypes),
                ("interfaces", nullable__type_list, __Type::Interfaces),
            ],
        );

        {
            let default_value = Some(self.input_values.push_value(SchemaInputValue::Boolean(false)));
            self.set_field_arguments(
                __type.id,
                "fields",
                std::iter::once(("includeDeprecated", nullable_boolean, default_value)),
            );
            self.set_field_arguments(
                __type.id,
                "enumValues",
                std::iter::once(("includeDeprecated", nullable_boolean, default_value)),
            );
        }

        let __input_value = self.insert_object_fields(
            __input_value,
            vec![
                ("name", required_string, __InputValue::Name),
                ("description", nullable_string, __InputValue::Description),
                // type added later
                ("defaultValue", nullable_string, __InputValue::DefaultValue),
                ("type", required__type, __InputValue::Type),
            ],
        );

        let __field = self.insert_object_fields(
            __field,
            vec![
                ("name", required_string, __Field::Name),
                ("description", nullable_string, __Field::Description),
                ("args", args, __Field::Args),
                // type added later
                ("isDeprecated", required_boolean, __Field::IsDeprecated),
                ("deprecationReason", nullable_string, __Field::DeprecationReason),
                ("type", required__type, __Field::Type),
            ],
        );

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
        let required__directive_list = Type {
            inner: __directive.id.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };
        let __schema = self.insert_object("__Schema");

        let __schema = self.insert_object_fields(
            __schema,
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

        let field_resolver = FieldResolver {
            resolver_id,
            field_requires: self.empty_requires_id,
        };
        /*
        __schema: __Schema!
        */
        let field_type_id = Type {
            inner: __schema.id.into(),
            wrapping: Wrapping::required(),
        };
        let [Some(__schema_field_id), Some(__type_field_id)] = ["__schema", "__type"].map(|name| {
            let fields = self[self.root_operation_types.query].fields;
            let idx = usize::from(fields.start)
                + self[fields]
                    .iter()
                    .position(|field| self.builder.strings[field.name] == name)?;
            Some(FieldDefinitionId::from(idx))
        }) else {
            panic!("Invariant broken: missing Query.__type or Query.__schema");
        };
        self[__schema_field_id].ty = field_type_id;
        self[__schema_field_id].resolvers.push(field_resolver.clone());

        /*
        __type(name: String!): __Type
        */
        let field_type_id = Type {
            inner: __type.id.into(),
            wrapping: Wrapping::nullable(),
        };
        self[__type_field_id].resolvers.push(field_resolver);
        self[__type_field_id].ty = field_type_id;

        self.set_field_arguments(
            self.root_operation_types.query,
            "__type",
            std::iter::once(("name", required_string, None)),
        );

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

        let values = if values.is_empty() {
            IdRange::empty()
        } else {
            let start_idx = self.enum_values.len();

            for value in values {
                let value = self.get_or_intern(value);
                self.enum_values.push(EnumValue {
                    name: value,
                    composed_directives: IdRange::empty(),
                    description: None,
                })
            }

            IdRange {
                start: EnumValueId::from(start_idx),
                end: EnumValueId::from(self.enum_values.len()),
            }
        };

        self.enums.push(crate::Enum {
            name,
            description: None,
            value_ids: values,
            composed_directives: IdRange::empty(),
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
            composed_directives: IdRange::empty(),
            cache_config: None,
            fields: IdRange::empty(),
        });
        ObjectId::from(self.objects.len() - 1)
    }

    fn insert_object_fields<E>(
        &mut self,
        object_id: ObjectId,
        fields: Vec<(&str, Type, E)>,
    ) -> IncompleteIntrospectionObject<E> {
        let start = self.field_definitions.len().into();
        let mut out_fields = Vec::new();

        for (name, r#type, tag) in fields {
            let id = self.field_definitions.len().into();
            let name = self.builder.strings.get_or_insert(name);

            self.field_definitions.push(FieldDefinition {
                name,
                ty: r#type,
                composed_directives: IdRange::empty(),
                resolvers: vec![],
                provides: vec![],
                argument_ids: IdRange::empty(),
                description: None,
                cache_config: None,
            });

            out_fields.push((id, tag));
        }

        let end = self.field_definitions.len().into();

        self[object_id].fields = IdRange { start, end };

        IncompleteIntrospectionObject {
            id: object_id,
            fields: out_fields,
        }
    }

    fn insert_object(&mut self, name: &str) -> ObjectId {
        let id = self.new_object(name);
        self.definitions.push(Definition::from(id));
        id
    }

    /// Warning: if you call this twice, the second call will overwrite the first.
    fn set_field_arguments<'b>(
        &mut self,
        object_id: ObjectId,
        field_name: &str,
        arguments: impl Iterator<Item = (&'b str, Type, Option<SchemaInputValueId>)>,
    ) {
        let fields = self[object_id].fields;
        let field_id = FieldDefinitionId::from(
            usize::from(fields.start)
                + self[fields]
                    .iter()
                    .position(|field| self.builder.strings[field.name] == field_name)
                    .expect("field to exist"),
        );
        let start = self.input_value_definitions.len();

        for (name, type_id, default_value) in arguments {
            self.insert_input_value(name, type_id, default_value);
        }

        let end = self.input_value_definitions.len();

        self[field_id].argument_ids = IdRange {
            start: start.into(),
            end: end.into(),
        };
    }

    fn insert_input_value(
        &mut self,
        name: &str,
        r#type: Type,
        default_value: Option<SchemaInputValueId>,
    ) -> InputValueDefinitionId {
        let name = self.get_or_intern(name);
        self.input_value_definitions.push(InputValueDefinition {
            name,
            description: None,
            default_value,
            ty: r#type,
        });
        InputValueDefinitionId::from(self.input_value_definitions.len() - 1)
    }

    fn field_type(&mut self, scalar_name: &str, scalar_type: ScalarType, wrapping: Wrapping) -> Type {
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
                    ty: scalar_type,
                    description: None,
                    specified_by_url: None,
                    composed_directives: IdRange::empty(),
                });
                ScalarId::from(self.scalars.len() - 1)
            }
        };
        let expected_kind = Definition::from(scalar_id);

        Type {
            inner: expected_kind,
            wrapping,
        }
    }

    fn get_or_intern(&mut self, value: &str) -> StringId {
        self.builder.strings.get_or_insert(value)
    }
}

struct IncompleteIntrospectionObject<E> {
    id: ObjectId,
    fields: Vec<(FieldDefinitionId, E)>,
}

impl<E: std::fmt::Debug, const N: usize> From<IncompleteIntrospectionObject<E>> for IntrospectionObject<E, N> {
    fn from(value: IncompleteIntrospectionObject<E>) -> Self {
        IntrospectionObject {
            id: value.id,
            fields: value.fields.try_into().unwrap(),
        }
    }
}
