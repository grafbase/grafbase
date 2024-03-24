use std::borrow::Cow;

use serde::Deserialize;
use wrapping::Wrapping;

use crate::{builder::BuildContext, EnumValueDefinition, InputValue, InputValueDefinition, Schema, Type};

use super::*;

fn create_schema_and_input_value() -> (Schema, SchemaInputValueId) {
    BuildContext::build_with(|ctx, graph| {
        graph.input_value_definitions.extend([
            InputValueDefinition {
                name: ctx.strings.get_or_insert("fieldA"),
                description: None,
                ty: Type {
                    inner: crate::Definition::Object(0.into()),
                    wrapping: Wrapping::new(false),
                }, // not used
                default_value: None,
            },
            InputValueDefinition {
                name: ctx.strings.get_or_insert("fieldB"),
                description: None,
                ty: Type {
                    inner: crate::Definition::Object(0.into()),
                    wrapping: Wrapping::new(false),
                }, // not used
                default_value: None,
            },
        ]);
        graph.enum_value_definitions.extend([
            EnumValueDefinition {
                name: ctx.strings.get_or_insert("ACTIVE"),
                description: None,
                composed_directives: Default::default(),
            },
            EnumValueDefinition {
                name: ctx.strings.get_or_insert("INACTIVE"),
                description: None,
                composed_directives: Default::default(),
            },
        ]);
        let list = graph.input_values.push_list(vec![
            SchemaInputValue::Null,
            SchemaInputValue::EnumValue(EnumValueDefinitionId::from(0)),
            SchemaInputValue::Int(73),
        ]);
        let input_fields = graph.input_values.push_input_object(vec![
            (
                InputValueDefinitionId::from(0),
                SchemaInputValue::EnumValue(EnumValueDefinitionId::from(1)),
            ),
            (
                InputValueDefinitionId::from(1),
                SchemaInputValue::String(ctx.strings.get_or_insert("some string value")),
            ),
        ]);
        let nested_fields = graph.input_values.push_map(vec![
            (ctx.strings.get_or_insert("null"), SchemaInputValue::Null),
            (
                ctx.strings.get_or_insert("string"),
                SchemaInputValue::String(ctx.strings.get_or_insert("some string value")),
            ),
            (
                ctx.strings.get_or_insert("enumValue"),
                SchemaInputValue::EnumValue(EnumValueDefinitionId::from(0)),
            ),
            (ctx.strings.get_or_insert("int"), SchemaInputValue::Int(7)),
            (ctx.strings.get_or_insert("bigInt"), SchemaInputValue::BigInt(8)),
            (ctx.strings.get_or_insert("u64"), SchemaInputValue::U64(9)),
            (ctx.strings.get_or_insert("float"), SchemaInputValue::Float(10.0)),
            (ctx.strings.get_or_insert("boolean"), SchemaInputValue::Boolean(true)),
        ]);
        let fields = graph.input_values.push_map(vec![
            (
                ctx.strings.get_or_insert("inputObject"),
                SchemaInputValue::InputObject(input_fields),
            ),
            (ctx.strings.get_or_insert("list"), SchemaInputValue::List(list)),
            (
                ctx.strings.get_or_insert("object"),
                SchemaInputValue::Map(nested_fields),
            ),
        ]);
        graph.input_values.push_value(SchemaInputValue::Map(fields))
    })
}

#[test]
fn test_display() {
    let (schema, id) = create_schema_and_input_value();
    let walker = schema.walk(&schema[id]);

    insta::assert_snapshot!(walker, @r###"{inputObject:{fieldA:INACTIVE,fieldB:"some string value"},list:[null,ACTIVE,73],object:{null:null,string:"some string value",enumValue:ACTIVE,int:7,bigInt:8,u64:9,float:10,boolean:true}}"###);
}

#[test]
fn test_serialize() {
    let (schema, id) = create_schema_and_input_value();
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
    let (schema, id) = create_schema_and_input_value();
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
    let (schema, id) = create_schema_and_input_value();
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
                            InputValueDefinition#0,
                            EnumValue(
                                EnumValue#1,
                            ),
                        ),
                        (
                            InputValueDefinition#1,
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
                            EnumValue#0,
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
                                EnumValue#0,
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
    let (schema, id) = create_schema_and_input_value();
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
