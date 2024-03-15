use crate::{
    EnumValueId, IdRange, InputValueDefinitionId, RawInputKeyValueId, RawInputObjectFieldValueId, RawInputValueId,
    SchemaInputValueId,
};

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
#[derive(Clone)]
pub struct RawInputValues<Str> {
    /// Inidividual input values and list values
    pub(super) values: Vec<RawInputValue<Str>>,
    /// InputObject's fields
    pub(super) input_fields: Vec<(InputValueDefinitionId, RawInputValue<Str>)>,
    /// Object's fields (for JSON)
    pub(super) key_values: Vec<(Str, RawInputValue<Str>)>,
}

impl<Str> Default for RawInputValues<Str> {
    fn default() -> Self {
        Self {
            // Reserve the first slot for undefined values.
            // This allows us in the engine to create from scratch a PlanInputValue that is
            // undefined. Used when requesting a field argument that wasn't provided.
            values: vec![RawInputValue::Undefined],
            input_fields: Default::default(),
            key_values: Default::default(),
        }
    }
}

/// Represents any possible input value for field arguments, operation variables and directive
/// arguments.
/// Used for storage and as input format for operation input values to support default values,
/// variables, etc.
#[derive(Debug, Clone)]
pub enum RawInputValue<Str> {
    Null,
    String(Str),
    EnumValue(EnumValueId),
    Int(i32),
    BigInt(i64),
    Float(f64),
    Boolean(bool),
    InputObject(IdRange<RawInputObjectFieldValueId<Str>>),
    List(IdRange<RawInputValueId<Str>>),

    /// for JSON
    Map(IdRange<RawInputKeyValueId<Str>>),
    U64(u64),

    /// Directive arguments have unknown enum values. Likely doesn't make sense, but not sure.
    UnknownEnumValue(Str),

    /// Primary purpose is for operation to reference the value of a variable which is initialized
    /// to the default or an arbitrary value first. It might also be used later to share the same
    /// InputValue at different places.
    Ref(RawInputValueId<Str>),

    /// Used to reference default values for operation input values. It's tricky without as default
    /// values also need to be taken into account for nested input object fields.
    SchemaRef(SchemaInputValueId),

    /// https://spec.graphql.org/October2021/#sec-Input-Objects.Input-Coercion
    /// An input { a: $var, b: 123 } without any variable should be interpreted as { b: 123 } if a
    /// is nullable.
    /// However for an operation variable we reserve an InputValue slot for its future value.
    /// Every place referencing it uses a Ref to it. So to ensure we do handle the undefiner case properly, we
    /// need a case for it.
    Undefined,
}

impl<Str> RawInputValues<Str> {
    pub fn undefined_value_id(&self) -> RawInputValueId<Str> {
        RawInputValueId::from(0)
    }

    pub fn push_value(&mut self, value: RawInputValue<Str>) -> RawInputValueId<Str> {
        let id = RawInputValueId::from(self.values.len());
        self.values.push(value);
        id
    }

    #[cfg(test)]
    pub fn push_list(&mut self, values: Vec<RawInputValue<Str>>) -> IdRange<RawInputValueId<Str>> {
        let start = self.values.len();
        self.values.extend(values);
        (start..self.values.len()).into()
    }

    /// Reserve InputValue slots for a list, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_list(&mut self, n: usize) -> IdRange<RawInputValueId<Str>> {
        let start = self.values.len();
        self.values.reserve(n);
        for _ in 0..n {
            self.values.push(RawInputValue::Null);
        }
        (start..self.values.len()).into()
    }

    #[cfg(test)]
    pub fn push_map(&mut self, fields: Vec<(Str, RawInputValue<Str>)>) -> IdRange<RawInputKeyValueId<Str>> {
        let start = self.key_values.len();
        self.key_values.extend(fields);
        (start..self.key_values.len()).into()
    }

    /// Reserve InputKeyValue slots for a map, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_map(&mut self, s: Str, n: usize) -> IdRange<RawInputKeyValueId<Str>>
    where
        Str: Clone,
    {
        let start = self.key_values.len();
        self.key_values.reserve(n);
        for _ in 0..n {
            self.key_values.push((s.clone(), RawInputValue::Null));
        }
        (start..self.key_values.len()).into()
    }

    pub fn push_input_object(
        &mut self,
        fields: impl IntoIterator<Item = (InputValueDefinitionId, RawInputValue<Str>)>,
    ) -> IdRange<RawInputObjectFieldValueId<Str>> {
        let start = self.input_fields.len();
        self.input_fields.extend(fields);
        (start..self.input_fields.len()).into()
    }

    /// Reserve InputObjectFieldValue slots for an InputObject, avoiding the need for an intermediate
    /// Vec to hold values as we need them to be contiguous.
    pub fn reserve_input_object(&mut self, n: usize) -> IdRange<RawInputObjectFieldValueId<Str>> {
        let start = self.input_fields.len();
        self.input_fields.reserve(n);
        for _ in 0..n {
            self.input_fields
                .push((InputValueDefinitionId::from(0), RawInputValue::Null));
        }
        (start..self.input_fields.len()).into()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use serde::Deserialize;

    use crate::{EnumValue, InputValue, InputValueDefinition, RawInputValuesContext, Schema, StringId, TypeId};

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
            RawInputValue::Null,
            RawInputValue::EnumValue(EnumValueId::from(0)),
            RawInputValue::Int(73),
        ]);
        let input_fields = schema.input_values.push_input_object(vec![
            (
                InputValueDefinitionId::from(0),
                RawInputValue::EnumValue(EnumValueId::from(1)),
            ),
            (
                InputValueDefinitionId::from(1),
                RawInputValue::String(StringId::from(1)),
            ),
        ]);
        let nested_fields = schema.input_values.push_map(vec![
            (StringId::from(6), RawInputValue::Null),
            (StringId::from(7), RawInputValue::String(StringId::from(1))),
            (StringId::from(8), RawInputValue::EnumValue(EnumValueId::from(0))),
            (StringId::from(9), RawInputValue::Int(7)),
            (StringId::from(10), RawInputValue::BigInt(8)),
            (StringId::from(11), RawInputValue::U64(9)),
            (StringId::from(12), RawInputValue::Float(10.0)),
            (StringId::from(13), RawInputValue::Boolean(true)),
        ]);
        let fields = schema.input_values.push_map(vec![
            (StringId::from(14), RawInputValue::InputObject(input_fields)),
            (StringId::from(15), RawInputValue::List(list)),
            (StringId::from(16), RawInputValue::Map(nested_fields)),
        ]);
        schema.input_values.push_value(RawInputValue::Map(fields));
        schema
    }

    #[test]
    fn test_display() {
        let schema = create_schema();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);
        let walker = RawInputValuesContext::walk(&schema.walker(), id);

        insta::assert_snapshot!(walker, @r###"{inputObject:{fieldA:INACTIVE,fieldB:"some string value"},list:[null,ACTIVE,73],object:{null:null,string:"some string value",enumValue:ACTIVE,int:7,bigInt:8,u64:9,float:10,boolean:true}}"###);
    }

    #[test]
    fn test_serialize() {
        let schema = create_schema();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);
        let walker = RawInputValuesContext::walk(&schema.walker(), id);

        insta::assert_json_snapshot!(walker, @r###"
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
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);
        let walker = RawInputValuesContext::walk(&schema.walker(), id);

        let value = serde_json::Value::deserialize(walker).unwrap();

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
    fn test_input_value() {
        let schema = create_schema();
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);
        let walker = RawInputValuesContext::walk(&schema.walker(), id);
        let input_value = InputValue::from(walker);

        insta::assert_debug_snapshot!(input_value, @r###"
        Map(
            [
                (
                    "inputObject",
                    InputObject(
                        [
                            (
                                0,
                                EnumValue(
                                    1,
                                ),
                            ),
                            (
                                1,
                                String(
                                    "some string value",
                                ),
                            ),
                        ],
                    ),
                ),
                (
                    "list",
                    List(
                        [
                            Null,
                            EnumValue(
                                0,
                            ),
                            Int(
                                73,
                            ),
                        ],
                    ),
                ),
                (
                    "object",
                    Map(
                        [
                            (
                                "null",
                                Null,
                            ),
                            (
                                "string",
                                String(
                                    "some string value",
                                ),
                            ),
                            (
                                "enumValue",
                                EnumValue(
                                    0,
                                ),
                            ),
                            (
                                "int",
                                Int(
                                    7,
                                ),
                            ),
                            (
                                "bigInt",
                                BigInt(
                                    8,
                                ),
                            ),
                            (
                                "u64",
                                U64(
                                    9,
                                ),
                            ),
                            (
                                "float",
                                Float(
                                    10.0,
                                ),
                            ),
                            (
                                "boolean",
                                Boolean(
                                    true,
                                ),
                            ),
                        ],
                    ),
                ),
            ],
        )
        "###);

        insta::assert_json_snapshot!(schema.walk(&input_value), @r###"
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
        let id = SchemaInputValueId::from(schema.input_values.values.len() - 1);
        let walker = RawInputValuesContext::walk(&schema.walker(), id);

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

        let input = Input::deserialize(walker).unwrap();

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

        serde::de::IgnoredAny::deserialize(walker).unwrap();
    }
}
