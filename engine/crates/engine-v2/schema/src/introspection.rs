use std::ops::{Deref, DerefMut};

use crate::{
    builder::BuildContext, DefinitionId, EntityDefinitionId, EnumDefinitionId, EnumDefinitionRecord, EnumValueId,
    EnumValueRecord, FieldDefinitionId, FieldDefinitionRecord, Graph, IdRange, InputValueDefinitionId,
    InputValueDefinitionRecord, ObjectDefinitionId, ObjectDefinitionRecord, ResolverDefinitionId,
    ResolverDefinitionRecord, ScalarDefinitionId, ScalarType, SchemaInputValueId, SchemaInputValueRecord, StringId,
    SubgraphId, TypeRecord, Wrapping,
};
use strum::EnumCount;

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IntrospectionResolverDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum IntrospectionField {
    Type,
    Schema,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::EnumCount, serde::Serialize, serde::Deserialize)]
pub enum __Schema {
    Description,
    Types,
    QueryType,
    MutationType,
    SubscriptionType,
    Directives,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumCount,
    serde::Serialize,
    serde::Deserialize,
)]
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

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumCount,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum __EnumValue {
    Name,
    Description,
    IsDeprecated,
    DeprecationReason,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumCount,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum __InputValue {
    Name,
    Description,
    Type,
    DefaultValue,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumCount,
    serde::Serialize,
    serde::Deserialize,
)]
// Using __Field conflicts with serde::Deserialize implementation
pub enum _Field {
    Name,
    Description,
    Args,
    Type,
    IsDeprecated,
    DeprecationReason,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum_macros::EnumCount,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum __Directive {
    Name,
    Description,
    Locations,
    Args,
    IsRepeatable,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IntrospectionMetadata {
    pub subgraph_id: SubgraphId,
    pub resolver_id: ResolverDefinitionId,
    pub meta_fields: [FieldDefinitionId; 2],
    pub type_kind: TypeKind,
    pub directive_location: DirectiveLocation,
    pub __schema: IntrospectionObject<__Schema, { __Schema::COUNT }>,
    pub __type: IntrospectionObject<__Type, { __Type::COUNT }>,
    pub __enum_value: IntrospectionObject<__EnumValue, { __EnumValue::COUNT }>,
    pub __input_value: IntrospectionObject<__InputValue, { __InputValue::COUNT }>,
    pub __field: IntrospectionObject<_Field, { _Field::COUNT }>,
    pub __directive: IntrospectionObject<__Directive, { __Directive::COUNT }>,
}

#[serde_with::serde_as]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(bound(serialize = "E: serde::Serialize", deserialize = "E: serde::Deserialize<'de>"))]
pub struct IntrospectionObject<E, const N: usize> {
    pub id: ObjectDefinitionId,
    #[serde_as(as = "[_; N]")]
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

impl IntrospectionMetadata {
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

#[derive(serde::Serialize, serde::Deserialize)]
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

#[derive(serde::Serialize, serde::Deserialize)]
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

pub(crate) struct IntrospectionBuilder<'a> {
    ctx: &'a mut BuildContext,
    graph: &'a mut Graph,
}

impl<'a> Deref for IntrospectionBuilder<'a> {
    type Target = Graph;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}

impl<'a> DerefMut for IntrospectionBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}

impl<'a> IntrospectionBuilder<'a> {
    pub fn create_data_source_and_insert_fields(
        ctx: &'a mut BuildContext,
        graph: &'a mut Graph,
    ) -> IntrospectionMetadata {
        Self { ctx, graph }.create_fields_and_insert_them()
    }

    #[allow(non_snake_case)]
    fn create_fields_and_insert_them(&mut self) -> IntrospectionMetadata {
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
            [
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

        let args = TypeRecord {
            definition_id: __input_value.into(),
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

        let locations = TypeRecord {
            definition_id: __directive_location.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };

        let __directive = self.insert_object_fields(
            __directive,
            [
                ("name", required_string, __Directive::Name),
                ("description", nullable_string, __Directive::Description),
                ("locations", locations, __Directive::Locations),
                ("args", args, __Directive::Args),
                ("isRepeatable", required_boolean, __Directive::IsRepeatable),
            ],
        );

        /*
        type __TypeRecord {
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

        let kind = TypeRecord {
            definition_id: __type_kind.into(),
            wrapping: Wrapping::required(),
        };
        let input_fields = TypeRecord {
            definition_id: __input_value.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };
        let nullable__field_list = TypeRecord {
            definition_id: __field.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };
        let nullable__enum_value_list = TypeRecord {
            definition_id: __enum_value.id.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };

        let required__type = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::required(),
        };
        let nullable__type = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::nullable(),
        };
        let required__type_list = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };
        let nullable__type_list = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::required().wrapped_by_nullable_list(),
        };

        let __type = self.insert_object_fields(
            __type,
            [
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
            let default_value = Some(
                self.graph
                    .input_values
                    .push_value(SchemaInputValueRecord::Boolean(false)),
            );
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
            [
                ("name", required_string, __InputValue::Name),
                ("description", nullable_string, __InputValue::Description),
                ("defaultValue", nullable_string, __InputValue::DefaultValue),
                ("type", required__type, __InputValue::Type),
            ],
        );

        let __field = self.insert_object_fields(
            __field,
            [
                ("name", required_string, _Field::Name),
                ("description", nullable_string, _Field::Description),
                ("args", args, _Field::Args),
                ("isDeprecated", required_boolean, _Field::IsDeprecated),
                ("deprecationReason", nullable_string, _Field::DeprecationReason),
                ("type", required__type, _Field::Type),
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
        let required__directive_list = TypeRecord {
            definition_id: __directive.id.into(),
            wrapping: Wrapping::required().wrapped_by_required_list(),
        };
        let __schema = self.insert_object("__Schema");

        let __schema = self.insert_object_fields(
            __schema,
            [
                ("description", nullable_string, __Schema::Description),
                ("types", required__type_list, __Schema::Types),
                ("queryType", required__type, __Schema::QueryType),
                ("mutationType", nullable__type, __Schema::MutationType),
                ("subscriptionType", nullable__type, __Schema::SubscriptionType),
                ("directives", required__directive_list, __Schema::Directives),
            ],
        );

        let resolver_id = ResolverDefinitionId::from(self.resolver_definitions.len());
        self.resolver_definitions.push(ResolverDefinitionRecord::Introspection);

        /*
        __schema: __Schema!
        */
        let field_type_id = TypeRecord {
            definition_id: __schema.id.into(),
            wrapping: Wrapping::required(),
        };
        let [Some(__schema_field_id), Some(__type_field_id)] = ["__schema", "__type"].map(|name| {
            let fields = self[self.root_operation_types_record.query_id].field_ids;
            let idx = usize::from(fields.start)
                + self[fields]
                    .iter()
                    .position(|field| self.ctx.strings[field.name_id] == name)?;
            Some(FieldDefinitionId::from(idx))
        }) else {
            panic!("Invariant broken: missing Query.__type or Query.__schema");
        };
        self[__schema_field_id].ty_record = field_type_id;
        self[__schema_field_id].resolver_ids.push(resolver_id);

        /*
        __type(name: String!): __Type
        */
        let field_type_id = TypeRecord {
            definition_id: __type.id.into(),
            wrapping: Wrapping::nullable(),
        };
        self[__type_field_id].ty_record = field_type_id;
        self[__type_field_id].resolver_ids.push(resolver_id);

        self.set_field_arguments(
            self.root_operation_types_record.query_id,
            "__type",
            std::iter::once(("name", required_string, None)),
        );

        // DataSource
        IntrospectionMetadata {
            subgraph_id: SubgraphId::Introspection,
            resolver_id,
            meta_fields: [__type_field_id, __schema_field_id],
            type_kind,
            directive_location,
            __schema,
            __type,
            __enum_value,
            __input_value,
            __field,
            __directive,
        }
    }

    fn insert_enum(&mut self, name: &str, values: &[&str]) -> EnumDefinitionId {
        let values = if values.is_empty() {
            IdRange::empty()
        } else {
            let start_idx = self.enum_value_definitions.len();

            for value in values {
                let name_id = self.get_or_intern(value);
                self.enum_value_definitions.push(EnumValueRecord {
                    name_id,
                    directive_ids: Vec::new(),
                    description_id: None,
                })
            }

            IdRange {
                start: EnumValueId::from(start_idx),
                end: EnumValueId::from(self.enum_value_definitions.len()),
            }
        };

        let name_id = self.get_or_intern(name);
        self.enum_definitions.push(EnumDefinitionRecord {
            name_id,
            description_id: None,
            value_ids: values,
            directive_ids: Vec::new(),
        });
        let enum_id = EnumDefinitionId::from(self.enum_definitions.len() - 1);
        self.type_definitions_ordered_by_name.push(DefinitionId::Enum(enum_id));
        enum_id
    }

    fn new_object(&mut self, name: &str) -> ObjectDefinitionId {
        let name_id = self.get_or_intern(name);
        self.object_definitions.push(ObjectDefinitionRecord {
            name_id,
            description_id: None,
            interface_ids: Vec::new(),
            directive_ids: Vec::new(),
            field_ids: IdRange::empty(),
        });
        ObjectDefinitionId::from(self.object_definitions.len() - 1)
    }

    fn insert_object_fields<E: std::fmt::Debug, const N: usize>(
        &mut self,
        object_id: ObjectDefinitionId,
        fields: [(&str, TypeRecord, E); N],
    ) -> IntrospectionObject<E, N> {
        let start = self.field_definitions.len().into();
        let mut out_fields = Vec::new();

        for (name, r#type, tag) in fields {
            let id = self.field_definitions.len().into();
            let name_id = self.ctx.strings.get_or_new(name);

            self.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id: None,
                ty_record: r#type,
                parent_entity_id: EntityDefinitionId::Object(object_id),
                only_resolvable_in_ids: vec![SubgraphId::Introspection],
                requires_records: Vec::new(),
                provides_records: Vec::new(),
                directive_ids: Vec::new(),
                resolver_ids: Vec::new(),
                argument_ids: IdRange::empty(),
            });

            out_fields.push((id, tag));
        }

        let end = self.field_definitions.len().into();

        self[object_id].field_ids = IdRange { start, end };

        IntrospectionObject {
            id: object_id,
            fields: out_fields.try_into().unwrap(),
        }
    }

    fn insert_object(&mut self, name: &str) -> ObjectDefinitionId {
        let id = self.new_object(name);
        self.type_definitions_ordered_by_name.push(DefinitionId::from(id));
        id
    }

    /// Warning: if you call this twice, the second call will overwrite the first.
    fn set_field_arguments<'b>(
        &mut self,
        object_id: ObjectDefinitionId,
        field_name: &str,
        arguments: impl Iterator<Item = (&'b str, TypeRecord, Option<SchemaInputValueId>)>,
    ) {
        let fields = self[object_id].field_ids;
        let field_id = FieldDefinitionId::from(
            usize::from(fields.start)
                + self[fields]
                    .iter()
                    .position(|field| self.ctx.strings[field.name_id] == field_name)
                    .expect("field to exist"),
        );
        let start = self.input_value_definitions.len();

        for (name, r#type, default_value) in arguments {
            self.insert_input_value(name, r#type, default_value);
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
        ty: TypeRecord,
        default_value_id: Option<SchemaInputValueId>,
    ) -> InputValueDefinitionId {
        let name_id = self.get_or_intern(name);
        self.input_value_definitions.push(InputValueDefinitionRecord {
            name_id,
            description_id: None,
            default_value_id,
            ty_record: ty,
            directive_ids: Vec::new(),
        });
        InputValueDefinitionId::from(self.input_value_definitions.len() - 1)
    }

    fn field_type(&mut self, scalar_name: &str, scalar_type: ScalarType, wrapping: Wrapping) -> TypeRecord {
        let scalar_id = match self
            .scalar_definitions
            .iter()
            .enumerate()
            .find(|(_, scalar)| self.ctx.strings[scalar.name_id] == scalar_name)
            .map(|(id, _)| ScalarDefinitionId::from(id))
        {
            Some(id) => id,
            None => {
                let name_id = self.ctx.strings.get_or_new(scalar_name);
                self.scalar_definitions.push(crate::ScalarDefinitionRecord {
                    name_id,
                    ty: scalar_type,
                    description_id: None,
                    specified_by_url_id: None,
                    directive_ids: Vec::new(),
                });
                ScalarDefinitionId::from(self.scalar_definitions.len() - 1)
            }
        };
        let expected_kind = DefinitionId::from(scalar_id);

        TypeRecord {
            definition_id: expected_kind,
            wrapping,
        }
    }

    fn get_or_intern(&mut self, value: &str) -> StringId {
        self.ctx.strings.get_or_new(value)
    }
}
