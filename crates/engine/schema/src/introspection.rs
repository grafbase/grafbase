use crate::{
    EntityDefinitionId, EnumDefinitionId, EnumDefinitionRecord, EnumValueId, EnumValueRecord, FieldDefinitionId,
    FieldDefinitionRecord, IdRange, InputValueDefinitionRecord, ObjectDefinitionId, ObjectDefinitionRecord,
    ResolverDefinitionId, ResolverDefinitionRecord, ScalarDefinitionId, ScalarType, SchemaInputValueId,
    SchemaInputValueRecord, StringId, SubgraphId, TypeDefinitionId, TypeRecord, Wrapping, builder::GraphBuilder,
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct IntrospectionSubgraph {
    pub resolver_definition_id: ResolverDefinitionId,
    pub meta_fields: [FieldDefinitionId; 2],
    pub meta_objects: [ObjectDefinitionId; 6],
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
#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

impl IntrospectionSubgraph {
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

impl GraphBuilder<'_> {
    #[allow(non_snake_case)]
    pub(crate) fn create_introspection_subgraph(&mut self) -> IntrospectionSubgraph {
        let nullable_string = self.field_type("String", ScalarType::String, Wrapping::default());
        let required_string = self.field_type("String", ScalarType::String, Wrapping::default().non_null());
        let required_boolean = self.field_type("Boolean", ScalarType::Boolean, Wrapping::default().non_null());
        let nullable_boolean = self.field_type("Boolean", ScalarType::Boolean, Wrapping::default());

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
            scalar: self.ingest_str("SCALAR"),
            object: self.ingest_str("OBJECT"),
            interface: self.ingest_str("INTERFACE"),
            union: self.ingest_str("UNION"),
            r#enum: self.ingest_str("ENUM"),
            input_object: self.ingest_str("INPUT_OBJECT"),
            list: self.ingest_str("LIST"),
            non_null: self.ingest_str("NON_NULL"),
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
            query: self.ingest_str("QUERY"),
            mutation: self.ingest_str("MUTATION"),
            subscription: self.ingest_str("SUBSCRIPTION"),
            field: self.ingest_str("FIELD"),
            fragment_definition: self.ingest_str("FRAGMENT_DEFINITION"),
            fragment_spread: self.ingest_str("FRAGMENT_SPREAD"),
            inline_fragment: self.ingest_str("INLINE_FRAGMENT"),
            variable_definition: self.ingest_str("VARIABLE_DEFINITION"),
            schema: self.ingest_str("SCHEMA"),
            scalar: self.ingest_str("SCALAR"),
            object: self.ingest_str("OBJECT"),
            field_definition: self.ingest_str("FIELD_DEFINITION"),
            argument_definition: self.ingest_str("ARGUMENT_DEFINITION"),
            interface: self.ingest_str("INTERFACE"),
            union: self.ingest_str("UNION"),
            r#enum: self.ingest_str("ENUM"),
            enum_value: self.ingest_str("ENUM_VALUE"),
            input_object: self.ingest_str("INPUT_OBJECT"),
            input_field_definition: self.ingest_str("INPUT_FIELD_DEFINITION"),
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
            wrapping: Wrapping::default().non_null().list_non_null(),
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
            wrapping: Wrapping::default().non_null().list_non_null(),
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
            wrapping: Wrapping::default().non_null(),
        };
        let input_fields = TypeRecord {
            definition_id: __input_value.into(),
            wrapping: Wrapping::default().non_null().list(),
        };
        let nullable__field_list = TypeRecord {
            definition_id: __field.into(),
            wrapping: Wrapping::default().non_null().list(),
        };
        let nullable__enum_value_list = TypeRecord {
            definition_id: __enum_value.id.into(),
            wrapping: Wrapping::default().non_null().list(),
        };

        let required__type = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::default().non_null(),
        };
        let nullable__type = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::default(),
        };
        let required__type_list = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::default().non_null().list_non_null(),
        };
        let nullable__type_list = TypeRecord {
            definition_id: __type.into(),
            wrapping: Wrapping::default().non_null().list(),
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
            wrapping: Wrapping::default().non_null().list_non_null(),
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

        let resolver_definition_id = ResolverDefinitionId::from(self.graph.resolver_definitions.len());
        self.graph
            .resolver_definitions
            .push(ResolverDefinitionRecord::Introspection);

        /*
        __schema: __Schema!
        */
        let field_type_id = TypeRecord {
            definition_id: __schema.id.into(),
            wrapping: Wrapping::default().non_null(),
        };
        let [Some(__schema_field_id), Some(__type_field_id)] = ["__schema", "__type"].map(|name| {
            self.graph[self.graph.root_operation_types_record.query_id]
                .field_ids
                .into_iter()
                .find(|id| self.ctx[self.graph[*id].name_id] == name)
        }) else {
            panic!("Invariant broken: missing Query.__type or Query.__schema");
        };
        self.graph[__schema_field_id].ty_record = field_type_id;
        self.graph[__schema_field_id].resolver_ids = vec![resolver_definition_id];

        /*
        __type(name: String!): __Type
        */
        let field_type_id = TypeRecord {
            definition_id: __type.id.into(),
            wrapping: Wrapping::default(),
        };
        self.graph[__type_field_id].ty_record = field_type_id;
        self.graph[__type_field_id].resolver_ids = vec![resolver_definition_id];

        self.set_field_arguments(
            self.graph.root_operation_types_record.query_id,
            "__type",
            std::iter::once(("name", required_string, None)),
        );

        // DataSource
        IntrospectionSubgraph {
            resolver_definition_id,
            meta_fields: [__type_field_id, __schema_field_id],
            meta_objects: [
                __schema.id,
                __type.id,
                __enum_value.id,
                __input_value.id,
                __field.id,
                __directive.id,
            ],
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
        let enum_id = EnumDefinitionId::from(self.graph.enum_definitions.len());
        let values = if values.is_empty() {
            IdRange::empty()
        } else {
            let start_idx = self.graph.enum_values.len();

            for value in values {
                let name_id = self.ingest_str(*value);
                self.graph.enum_values.push(EnumValueRecord {
                    name_id,
                    parent_enum_id: enum_id,
                    directive_ids: Vec::new(),
                    description_id: None,
                })
            }

            IdRange {
                start: EnumValueId::from(start_idx),
                end: EnumValueId::from(self.graph.enum_values.len()),
            }
        };

        let name_id = self.ingest_str(name);
        self.graph.enum_definitions.push(EnumDefinitionRecord {
            name_id,
            description_id: None,
            value_ids: values,
            directive_ids: Vec::new(),
            exists_in_subgraph_ids: vec![SubgraphId::Introspection],
        });

        EnumDefinitionId::from(self.graph.enum_definitions.len() - 1)
    }

    fn insert_object(&mut self, name: &str) -> ObjectDefinitionId {
        let name_id = self.ingest_str(name);
        self.graph.object_definitions.push(ObjectDefinitionRecord {
            name_id,
            description_id: None,
            interface_ids: Vec::new(),
            directive_ids: Vec::new(),
            field_ids: IdRange::empty(),
            join_implement_records: Vec::new(),
            exists_in_subgraph_ids: vec![SubgraphId::Introspection],
        });
        ObjectDefinitionId::from(self.graph.object_definitions.len() - 1)
    }

    fn insert_object_fields<E: std::fmt::Debug, const N: usize>(
        &mut self,
        object_id: ObjectDefinitionId,
        fields: [(&str, TypeRecord, E); N],
    ) -> IntrospectionObject<E, N> {
        let start = self.graph.field_definitions.len().into();
        let mut out_fields = Vec::new();

        for (name, r#type, tag) in fields {
            let id = self.graph.field_definitions.len().into();
            let name_id = self.ingest_str(name);

            self.graph.field_definitions.push(FieldDefinitionRecord {
                name_id,
                description_id: None,
                ty_record: r#type,
                parent_entity_id: EntityDefinitionId::Object(object_id),
                exists_in_subgraph_ids: vec![SubgraphId::Introspection],
                requires_records: Vec::new(),
                provides_records: Vec::new(),
                directive_ids: Vec::new(),
                resolver_ids: Vec::new(),
                argument_ids: IdRange::empty(),
                subgraph_type_records: Vec::new(),
                derive_ids: Default::default(),
            });

            out_fields.push((id, tag));
        }

        let end = self.graph.field_definitions.len().into();

        self.graph[object_id].field_ids = IdRange { start, end };

        IntrospectionObject {
            id: object_id,
            fields: out_fields.try_into().unwrap(),
        }
    }

    /// Warning: if you call this twice, the second call will overwrite the first.
    fn set_field_arguments<'b>(
        &mut self,
        object_id: ObjectDefinitionId,
        field_name: &str,
        arguments: impl Iterator<Item = (&'b str, TypeRecord, Option<SchemaInputValueId>)>,
    ) {
        let fields = self.graph[object_id].field_ids;
        let field_id = FieldDefinitionId::from(
            usize::from(fields.start)
                + self.graph[fields]
                    .iter()
                    .position(|field| self.ctx[field.name_id] == field_name)
                    .expect("field to exist"),
        );
        let start = self.graph.input_value_definitions.len();

        for (name, ty_record, default_value_id) in arguments {
            let name_id = self.ingest_str(name);
            self.graph.input_value_definitions.push(InputValueDefinitionRecord {
                name_id,
                description_id: None,
                default_value_id,
                parent_id: field_id.into(),
                ty_record,
                directive_ids: Vec::new(),
                is_internal_in_id: None,
            });
        }

        let end = self.graph.input_value_definitions.len();

        self.graph[field_id].argument_ids = IdRange {
            start: start.into(),
            end: end.into(),
        };
    }

    fn field_type(&mut self, scalar_name: &str, scalar_type: ScalarType, wrapping: Wrapping) -> TypeRecord {
        let scalar_id = match self
            .graph
            .scalar_definitions
            .iter()
            .enumerate()
            .find(|(_, scalar)| self.ctx[scalar.name_id] == scalar_name)
            .map(|(id, _)| ScalarDefinitionId::from(id))
        {
            Some(id) => id,
            None => {
                let name_id = self.ingest_str(scalar_name);
                self.graph.scalar_definitions.push(crate::ScalarDefinitionRecord {
                    name_id,
                    ty: scalar_type,
                    description_id: None,
                    specified_by_url_id: None,
                    directive_ids: Vec::new(),
                    exists_in_subgraph_ids: vec![SubgraphId::Introspection],
                });
                ScalarDefinitionId::from(self.graph.scalar_definitions.len() - 1)
            }
        };
        let expected_kind = TypeDefinitionId::from(scalar_id);

        TypeRecord {
            definition_id: expected_kind,
            wrapping,
        }
    }
}
