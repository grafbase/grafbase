use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use crate::{
    Definition, EnumId, EnumValue, Field, FieldArgument, FieldId, FieldType, FieldTypeId, ObjectField, ObjectId,
    ScalarId, Schema, StringId, Value, Wrapping,
};

pub struct IntrospectionFields<'a> {
    schema: &'a mut Schema,
    strings_map: HashMap<String, StringId>,
}

impl<'a> Deref for IntrospectionFields<'a> {
    type Target = Schema;
    fn deref(&self) -> &Self::Target {
        self.schema
    }
}

impl<'a> DerefMut for IntrospectionFields<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.schema
    }
}

impl<'a> IntrospectionFields<'a> {
    // /!\ You MUST call schema.ensure_proper_ordering() after calling this function. /!\
    pub fn insert_into(schema: &'a mut Schema) {
        let strings_map = schema
            .strings
            .iter()
            .enumerate()
            .map(|(id, s)| (s.to_string(), StringId::from(id)))
            .collect();
        Self { schema, strings_map }.create_fields_and_insert_them();
    }

    #[allow(non_snake_case)]
    fn create_fields_and_insert_them(&mut self) {
        let nullable_string = self.find_or_create_field_type("String", Wrapping::nullable());
        let required_string = self.find_or_create_field_type("String", Wrapping::required());
        let required_boolean = self.find_or_create_field_type("Boolean", Wrapping::required());
        let nullable_boolean = self.find_or_create_field_type("Boolean", Wrapping::nullable());

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
            &[
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
            &[
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
            &[
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
                &[
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
                &[
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
                let field_id = self.insert_field("fields", nullable__field_list);
                let name = self.get_or_intern("includeDeprecated");
                self[field_id].arguments.push(FieldArgument {
                    name,
                    default_value: Some(Value::Boolean(false)),
                    type_id: nullable_boolean,
                });
                self.object_fields.push(ObjectField {
                    object_id: __type,
                    field_id,
                });
            }
            {
                let nullable__enum_value_list =
                    self.insert_field_type(__enum_value, Wrapping::required().nullable_list());
                let field_id = self.insert_field("enumValues", nullable__enum_value_list);
                let name = self.get_or_intern("includeDeprecated");
                self[field_id].arguments.push(FieldArgument {
                    name,
                    default_value: Some(Value::Boolean(false)),
                    type_id: nullable_boolean,
                });
                self.object_fields.push(ObjectField {
                    object_id: __type,
                    field_id,
                });
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
        self.insert_object(
            "__Schema",
            &[
                ("description", nullable_string),
                ("types", required__type_list),
                ("queryType", required__type),
                ("mutationType", nullable__type),
                ("subscriptionType", nullable__type),
                ("directives", required__directive_list),
            ],
        );
    }

    fn insert_enum(&mut self, name: &str, values: &[&str]) -> EnumId {
        let name = self.get_or_intern(name);
        let values = values
            .iter()
            .map(|value| {
                let value = self.get_or_intern(value);
                EnumValue {
                    value,
                    composed_directives: vec![],
                }
            })
            .collect();

        self.enums.push(crate::Enum {
            name,
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
            implements_interfaces: vec![],
            resolvable_keys: vec![],
            composed_directives: vec![],
        });
        ObjectId::from(self.objects.len() - 1)
    }

    fn insert_object_field(&mut self, object_id: ObjectId, name: &str, field_type_id: FieldTypeId) {
        let field_id = self.insert_field(name, field_type_id);
        self.object_fields.push(ObjectField { object_id, field_id });
    }

    fn insert_field(&mut self, name: &str, field_type_id: FieldTypeId) -> FieldId {
        let name = self.get_or_intern(name);
        self.fields.push(Field {
            name,
            field_type_id,
            composed_directives: vec![],
            resolvers: vec![],
            provides: vec![],
            arguments: vec![],
        });
        FieldId::from(self.fields.len() - 1)
    }

    fn insert_object(&mut self, name: &str, fields: &[(&str, FieldTypeId)]) -> ObjectId {
        let object_id = self.new_object(name);
        self.definitions.push(Definition::from(object_id));
        let field_ids: Vec<_> = fields
            .iter()
            .map(|(name, field_type_id)| self.insert_field(name, *field_type_id))
            .map(|field_id| ObjectField { object_id, field_id })
            .collect();
        self.object_fields.extend(field_ids);
        object_id
    }

    fn insert_field_type(&mut self, kind: impl Into<Definition>, wrapping: Wrapping) -> FieldTypeId {
        self.field_types.push(FieldType {
            kind: kind.into(),
            wrapping,
        });
        FieldTypeId::from(self.field_types.len() - 1)
    }

    fn find_scalar_id(&self, name: &str) -> ScalarId {
        self.scalars
            .iter()
            .enumerate()
            .find(|(_, scalar)| self[scalar.name] == name)
            .map(|(id, _)| ScalarId::from(id))
            .unwrap()
    }

    fn find_or_create_field_type(&mut self, scalar_name: &str, expected_wrapping: Wrapping) -> FieldTypeId {
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
                    composed_directives: vec![],
                });
                ScalarId::from(self.scalars.len() - 1)
            }
        };
        let expected_kind = Definition::from(scalar_id);
        match self
            .field_types
            .iter()
            .enumerate()
            .find(|(_, FieldType { kind, wrapping })| kind == &expected_kind && wrapping == &expected_wrapping)
        {
            Some((id, _)) => FieldTypeId::from(id),
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
