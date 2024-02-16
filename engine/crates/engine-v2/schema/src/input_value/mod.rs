use crate::{EnumValueId, IdRange, InputValueDefinitionId, SchemaWalker, StringId};

mod de;
mod display;
mod error;
mod ids;
mod ser;

pub use error::*;
pub use ids::*;

pub type SchemaInputValues = InputValues<StringId>;
pub type SchemaInputValue = InputValue<StringId>;
pub type SchemaInputValueId = InputValueId<StringId>;
pub type SchemaInputObjectFieldValueId = InputObjectFieldValueId<StringId>;
pub type SchemaInputKeyValueId = InputKeyValueId<StringId>;
pub type SchemaInputMap = IdRange<InputKeyValueId<StringId>>;

/// Holds input values for the Schema and for an Operation during execution.
/// It's generic over 'Str', the string representation:
/// - StringId for the Schema (default input values and directive arguments)
/// - Box<str> for the Operation
///
/// This allow us to share the code for:
/// - Display: used to print default values in introspection and add input values in subgraph query strings.
/// - Serialize: used to serialize variables/arguments.
/// - Deserializer: used by resolvers to deserialize arguments into a specific struct.
///
/// Data is stored in flat arrays for hopefully faster serialization and as as bonus smaller InputValue
/// footprint on x64 architectures: u64 + usize (padding) rather than 3 * usize (Box<[]> + padding).
pub struct InputValues<Str> {
    /// Inidividual input values and list values
    pub values: Vec<InputValue<Str>>,
    /// InputObject's fields
    pub input_fields: Vec<(InputValueDefinitionId, InputValue<Str>)>,
    /// Object's fields (for JSON)
    pub key_values: Vec<(Str, InputValue<Str>)>,
}

impl<Str> Default for InputValues<Str> {
    fn default() -> Self {
        Self {
            input_fields: Default::default(),
            values: Default::default(),
            key_values: Default::default(),
        }
    }
}

/// Represents any possible input value for field arguments, operation variables and directive
/// arguments.
#[derive(Debug, Clone)]
pub enum InputValue<Str> {
    Null,
    String(Str),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    InputObject(IdRange<InputObjectFieldValueId<Str>>),
    List(IdRange<InputValueId<Str>>),

    // for JSON
    Map(IdRange<InputKeyValueId<Str>>),
    U64(u64),

    // Directive arguments have unknown enum values. Likely doesn't make sense, but not sure.
    UnknownEnumValue(Str),
}

impl<Str> InputValues<Str> {
    pub fn push_value(&mut self, value: InputValue<Str>) -> InputValueId<Str> {
        let id = InputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    pub fn push_list(&mut self, values: Vec<InputValue<Str>>) -> IdRange<InputValueId<Str>> {
        let start = self.values.len();
        self.values.extend(values);
        (start..self.values.len()).into()
    }

    pub fn push_map(&mut self, fields: Vec<(Str, InputValue<Str>)>) -> IdRange<InputKeyValueId<Str>> {
        let start = self.key_values.len();
        self.key_values.extend(fields);
        (start..self.key_values.len()).into()
    }

    pub fn push_input_object(
        &mut self,
        fields: Vec<(InputValueDefinitionId, InputValue<Str>)>,
    ) -> IdRange<InputObjectFieldValueId<Str>> {
        let start = self.input_fields.len();
        self.input_fields.extend(fields);
        (start..self.input_fields.len()).into()
    }
}

pub trait InputValuesContext<'ctx, Str>: Clone + Copy + 'ctx {
    fn schema_walker(&self) -> &SchemaWalker<'ctx, ()>;
    fn get_str(&self, s: &Str) -> &'ctx str;
    fn input_values(&self) -> &'ctx InputValues<Str>;

    fn input_value_as_serializable(&self, id: InputValueId<Str>) -> impl serde::Serialize + 'ctx
    where
        Str: 'ctx,
    {
        ser::SerializableInputValue {
            ctx: *self,
            value: &self.input_values()[id],
        }
    }

    fn input_value_as_deserializer(&self, id: InputValueId<Str>) -> impl serde::Deserializer<'ctx> + 'ctx
    where
        Str: 'ctx,
    {
        de::InputValueDeserializer {
            ctx: *self,
            value: &self.input_values()[id],
        }
    }

    /// Display the input value with GraphQL syntax.
    fn input_value_as_graphql_display(&self, id: InputValueId<Str>) -> impl std::fmt::Display + 'ctx
    where
        Str: 'ctx,
    {
        display::GraphqlDisplayableInpuValue {
            ctx: *self,
            value: &self.input_values()[id],
        }
    }
}

impl<'ctx> InputValuesContext<'ctx, StringId> for SchemaWalker<'ctx, ()> {
    fn schema_walker(&self) -> &SchemaWalker<'ctx, ()> {
        self
    }

    fn get_str(&self, s: &StringId) -> &'ctx str {
        &self.schema[*s]
    }

    fn input_values(&self) -> &'ctx InputValues<StringId> {
        &self.schema.input_values
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use serde::Deserialize;

    use crate::{EnumValue, InputValueDefinition, Schema, TypeId};

    use super::*;

    fn create_schema() -> Schema {
        let mut schema = Schema::empty();
        schema.input_value_definitions.extend([
            InputValueDefinition {
                name: StringId::from(4),
                description: None,
                type_id: TypeId::from(0), // not used
                default_value: None,
            },
            InputValueDefinition {
                name: StringId::from(5),
                description: None,
                type_id: TypeId::from(0), // not used
                default_value: None,
            },
        ]);
        schema.enum_values.extend([
            EnumValue {
                name: StringId::from(2),
                description: None,
                composed_directives: Default::default(),
            },
            EnumValue {
                name: StringId::from(3),
                description: None,
                composed_directives: Default::default(),
            },
        ]);
        schema.strings.extend([
            "some string value".to_string(), // 1
            "ACTIVE".to_string(),            // 2
            "INACTIVE".to_string(),          // 3
            "fieldA".to_string(),            // 4
            "fieldB".to_string(),            // 5
            // ---
            "null".to_string(),        // 6
            "string".to_string(),      // 7
            "enumValue".to_string(),   // 8
            "int".to_string(),         // 9
            "bigInt".to_string(),      // 10
            "u64".to_string(),         // 11
            "float".to_string(),       // 12
            "boolean".to_string(),     // 13
            "inputObject".to_string(), // 14
            "list".to_string(),        // 15
            "object".to_string(),      // 16
        ]);
        let list = schema.input_values.push_list(vec![
            InputValue::Null,
            InputValue::EnumValue(EnumValueId::from(0)),
            InputValue::Int(73),
        ]);
        let input_fields = schema.input_values.push_input_object(vec![
            (
                InputValueDefinitionId::from(0),
                InputValue::EnumValue(EnumValueId::from(1)),
            ),
            (InputValueDefinitionId::from(1), InputValue::String(StringId::from(1))),
        ]);
        let nested_fields = schema.input_values.push_map(vec![
            (StringId::from(6), InputValue::Null),
            (StringId::from(7), InputValue::String(StringId::from(1))),
            (StringId::from(8), InputValue::EnumValue(EnumValueId::from(0))),
            (StringId::from(9), InputValue::Int(7)),
            (StringId::from(10), InputValue::BigInt(8)),
            (StringId::from(11), InputValue::U64(9)),
            (StringId::from(12), InputValue::Float(10.0)),
            (StringId::from(13), InputValue::Boolean(true)),
        ]);
        let fields = schema.input_values.push_map(vec![
            (StringId::from(14), InputValue::InputObject(input_fields)),
            (StringId::from(15), InputValue::List(list)),
            (StringId::from(16), InputValue::Map(nested_fields)),
        ]);
        schema.input_values.push_value(InputValue::Map(fields));
        schema
    }

    #[test]
    fn test_display() {
        let schema = create_schema();
        let walker = schema.walker();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);

        insta::assert_display_snapshot!(walker.input_value_as_graphql_display(id), @r###"{inputObject:{fieldA:INACTIVE,fieldB:"some string value"},list:[null,ACTIVE,73],object:{null:null,string:"some string value",enumValue:ACTIVE,int:7,bigInt:8,u64:9,float:10,boolean:true}}"###);
    }

    #[test]
    fn test_serialize() {
        let schema = create_schema();
        let walker = schema.walker();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);

        insta::assert_json_snapshot!(walker.input_value_as_serializable(id), @r###"
        {
          "inputObject": {
            "fieldA": "INACTIVE",
            "fieldB": "some string value"
          },
          "list": [
            null,
            "ACTIVE",
            73
          ],
          "object": {
            "null": null,
            "string": "some string value",
            "enumValue": "ACTIVE",
            "int": 7,
            "bigInt": 8,
            "u64": 9,
            "float": 10.0,
            "boolean": true
          }
        }
        "###);
    }

    #[test]
    fn test_deserializer() {
        let schema = create_schema();
        let walker = schema.walker();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);

        let value = serde_json::Value::deserialize(walker.input_value_as_deserializer(id)).unwrap();

        insta::assert_json_snapshot!(value, @r###"
        {
          "inputObject": {
            "fieldA": "INACTIVE",
            "fieldB": "some string value"
          },
          "list": [
            null,
            "ACTIVE",
            73
          ],
          "object": {
            "null": null,
            "string": "some string value",
            "enumValue": "ACTIVE",
            "int": 7,
            "bigInt": 8,
            "u64": 9,
            "float": 10.0,
            "boolean": true
          }
        }
        "###);
    }

    #[test]
    fn test_struct_deserializer() {
        let schema = create_schema();
        let walker = schema.walker();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);

        #[allow(unused)]
        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct InputObject<'a> {
            #[serde(borrow)]
            field_a: Cow<'a, str>,
            field_b: &'a str,
        }

        #[allow(unused)]
        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            null: Option<String>,
            string: String,
            enum_value: Option<String>,
            int: i32,
            big_int: i64,
            u64: u64,
            float: f64,
            boolean: bool,
        }

        #[allow(unused)]
        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Input<'a> {
            #[serde(borrow)]
            input_object: InputObject<'a>,
            list: Vec<serde_json::Value>,
            object: Object,
        }

        let input = Input::deserialize(walker.input_value_as_deserializer(id)).unwrap();

        insta::assert_debug_snapshot!(input, @r###"
        Input {
            input_object: InputObject {
                field_a: "INACTIVE",
                field_b: "some string value",
            },
            list: [
                Null,
                String("ACTIVE"),
                Number(73),
            ],
            object: Object {
                null: None,
                string: "some string value",
                enum_value: Some(
                    "ACTIVE",
                ),
                int: 7,
                big_int: 8,
                u64: 9,
                float: 10.0,
                boolean: true,
            },
        }
        "###);

        serde::de::IgnoredAny::deserialize(walker.input_value_as_deserializer(id)).unwrap();
    }
}
