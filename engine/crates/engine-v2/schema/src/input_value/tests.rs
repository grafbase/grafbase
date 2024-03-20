use std::borrow::Cow;

use serde::Deserialize;
use wrapping::Wrapping;

use crate::{EnumValue, InputValue, InputValueDefinition, Schema, StringId, Type};

use super::*;

fn create_schema() -> Schema {
    let mut schema = Schema::empty();
    schema.input_value_definitions.extend([
        InputValueDefinition {
            name: StringId::from(4),
            description: None,
            r#type: Type {
                inner: crate::Definition::Object(0.into()),
                wrapping: Wrapping::new(false),
            }, // not used
            default_value: None,
        },
        InputValueDefinition {
            name: StringId::from(5),
            description: None,
            r#type: Type {
                inner: crate::Definition::Object(0.into()),
                wrapping: Wrapping::new(false),
            }, // not used
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
    let list = schema.default_input_values.push_list(vec![
        SchemaInputValue::Null,
        SchemaInputValue::EnumValue(EnumValueId::from(0)),
        SchemaInputValue::Int(73),
    ]);
    let input_fields = schema.default_input_values.push_input_object(vec![
        (
            InputValueDefinitionId::from(0),
            SchemaInputValue::EnumValue(EnumValueId::from(1)),
        ),
        (
            InputValueDefinitionId::from(1),
            SchemaInputValue::String(StringId::from(1)),
        ),
    ]);
    let nested_fields = schema.default_input_values.push_map(vec![
        (StringId::from(6), SchemaInputValue::Null),
        (StringId::from(7), SchemaInputValue::String(StringId::from(1))),
        (StringId::from(8), SchemaInputValue::EnumValue(EnumValueId::from(0))),
        (StringId::from(9), SchemaInputValue::Int(7)),
        (StringId::from(10), SchemaInputValue::BigInt(8)),
        (StringId::from(11), SchemaInputValue::U64(9)),
        (StringId::from(12), SchemaInputValue::Float(10.0)),
        (StringId::from(13), SchemaInputValue::Boolean(true)),
    ]);
    let fields = schema.default_input_values.push_map(vec![
        (StringId::from(14), SchemaInputValue::InputObject(input_fields)),
        (StringId::from(15), SchemaInputValue::List(list)),
        (StringId::from(16), SchemaInputValue::Map(nested_fields)),
    ]);
    schema.default_input_values.push_value(SchemaInputValue::Map(fields));
    schema
}

#[test]
fn test_display() {
    let schema = create_schema();
    let id = SchemaInputValueId::from(schema.default_input_values.values.len() - 1);
    let walker = schema.walk(&schema[id]);

    insta::assert_snapshot!(walker, @r###"{inputObject:{fieldA:INACTIVE,fieldB:"some string value"},list:[null,ACTIVE,73],object:{null:null,string:"some string value",enumValue:ACTIVE,int:7,bigInt:8,u64:9,float:10,boolean:true}}"###);
}

#[test]
fn test_serialize() {
    let schema = create_schema();
    let id = SchemaInputValueId::from(schema.default_input_values.values.len() - 1);
    let walker = schema.walk(&schema[id]);

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
    let id = SchemaInputValueId::from(schema.default_input_values.values.len() - 1);
    let walker = schema.walk(&schema[id]);

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
    let id = SchemaInputValueId::from(schema.default_input_values.values.len() - 1);
    let walker = schema.walk(&schema[id]);
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
    let id = SchemaInputValueId::from(schema.default_input_values.values.len() - 1);
    let walker = schema.walk(&schema[id]);

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
