use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    DataType, Definition, EnumId, EnumValue, Field, FieldId, InputValue, InputValueId, ObjectField, ObjectId, ScalarId,
    Schema, SchemaWalker, StringId, Type, TypeId, Value, Wrapping,
};

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
    metadata: Option<Metadata>,
}

pub struct Metadata {
    pub meta_fields: [FieldId; 2],
    pub type_kind: TypeKind,
    pub directive_location: DirectiveLocation,
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

pub struct Introspection {
    schema: Schema,
    strings_map: HashMap<String, StringId>,
}

impl Deref for Introspection {
    type Target = Schema;
    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}

impl DerefMut for Introspection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.schema
    }
}

impl Introspection {
    pub fn finalize_schema(schema: Schema) -> Schema {
        let strings_map = schema
            .strings
            .iter()
            .enumerate()
            .map(|(id, s)| (s.to_string(), StringId::from(id)))
            .collect();
        let mut inserter = Self { schema, strings_map };
        inserter.create_fields_and_insert_them();
        inserter.schema.finalize()
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
                ("name", required_string),
                ("description", nullable_string),
                ("isDeprecated", required_boolean),
                ("deprecationReason", nullable_string),
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
        let __input_value = self.insert_object(
            "__InputValue",
            vec![
                ("name", required_string),
                ("description", nullable_string),
                // type added later
                ("defaultValue", nullable_string),
            ],
        );
        let args = self.insert_field_type(__input_value, Wrapping::required().required_list());

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
        let __field = self.insert_object(
            "__Field",
            vec![
                ("name", required_string),
                ("description", nullable_string),
                ("args", args),
                // type added later
                ("isDeprecated", required_boolean),
                ("deprecationReason", nullable_string),
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
            let locations = self.insert_field_type(__directive_location, Wrapping::required().required_list());
            self.insert_object(
                "__Directive",
                vec![
                    ("name", required_string),
                    ("description", nullable_string),
                    ("locations", locations),
                    ("args", args),
                    ("isRepeatable", required_boolean),
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
        let __type = {
            let kind = self.insert_field_type(__type_kind, Wrapping::required());
            let input_fields = self.insert_field_type(__input_value, Wrapping::required().nullable_list());
            let __type = self.insert_object(
                "__Type",
                vec![
                    ("kind", kind),
                    ("name", nullable_string),
                    ("description", nullable_string),
                    ("inputFields", input_fields),
                    ("specifiedByURL", nullable_string),
                    // other fields added later
                ],
            );
            {
                let nullable__field_list = self.insert_field_type(__field, Wrapping::required().nullable_list());
                let field_id = self.insert_object_field(__type, "fields", nullable__field_list);
                let input_value_id =
                    self.insert_input_value("includeDeprecated", nullable_boolean, Some(Value::Boolean(false)));
                self[field_id].arguments.push(input_value_id);
            }
            {
                let nullable__enum_value_list =
                    self.insert_field_type(__enum_value, Wrapping::required().nullable_list());
                let field_id = self.insert_object_field(__type, "enumValues", nullable__enum_value_list);
                let input_value_id =
                    self.insert_input_value("includeDeprecated", nullable_boolean, Some(Value::Boolean(false)));
                self[field_id].arguments.push(input_value_id);
            }
            __type
        };

        let required__type = self.insert_field_type(__type, Wrapping::required());
        let nullable__type = self.insert_field_type(__type, Wrapping::nullable());
        let required__type_list = self.insert_field_type(__type, Wrapping::required().required_list());
        let nullable__type_list = self.insert_field_type(__type, Wrapping::required().nullable_list());

        self.insert_object_field(__input_value, "type", required__type);
        self.insert_object_field(__field, "type", required__type);
        self.insert_object_field(__type, "ofType", nullable__type);
        self.insert_object_field(__type, "possibleTypes", nullable__type_list);
        self.insert_object_field(__type, "interfaces", nullable__type_list);

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
        let required__directive_list = self.insert_field_type(__directive, Wrapping::required().required_list());
        let __schema = self.insert_object(
            "__Schema",
            vec![
                ("description", nullable_string),
                ("types", required__type_list),
                ("queryType", required__type),
                ("mutationType", nullable__type),
                ("subscriptionType", nullable__type),
                ("directives", required__directive_list),
            ],
        );

        /*
        __schema: __Schema!
        */
        let field_type_id = self.insert_field_type(__schema, Wrapping::required());
        let __schema_field_id = self.insert_object_field(self.root_operation_types.query, "__schema", field_type_id);

        /*
        __type(name: String!): __Type
        */
        let field_type_id = self.insert_field_type(__type, Wrapping::nullable());
        let __type_field_id = self.insert_object_field(self.root_operation_types.query, "__type", field_type_id);
        let input_value_id = self.insert_input_value("name", required_string, None);
        self[__type_field_id].arguments.push(input_value_id);

        // DataSource
        self.data_sources.introspection.metadata = Some(Metadata {
            meta_fields: [__type_field_id, __schema_field_id],
            type_kind,
            directive_location,
        });

        // Introspection resolver is used as the default one which allows to also handle the query
        // `query { __typename }` in a more natural way.
        self.resolvers.push(crate::Resolver::Introspection(Resolver));
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

    fn insert_object(&mut self, name: &str, fields: Vec<(&str, TypeId)>) -> ObjectId {
        let object_id = self.new_object(name);
        self.definitions.push(Definition::from(object_id));
        for (name, field_type_id) in fields {
            self.insert_object_field(object_id, name, field_type_id);
        }
        object_id
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
            .find(|(_, scalar)| self[scalar.name] == scalar_name)
            .map(|(id, _)| ScalarId::from(id))
        {
            Some(id) => id,
            None => {
                let name = self.get_or_intern(scalar_name);
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
        *(self.strings_map.entry(value.to_string()).or_insert_with_key(|key| {
            self.schema.strings.push(key.to_string());
            StringId::from(self.schema.strings.len() - 1)
        }))
    }
}
