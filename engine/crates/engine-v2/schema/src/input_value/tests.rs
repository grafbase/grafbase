use std::borrow::Cow;

use serde::Deserialize;
use wrapping::Wrapping;

use crate::{builder::BuildContext, EnumValueRecord, InputValue, InputValueDefinitionRecord, Schema, TypeRecord};

use super::*;

fn create_schema_and_input_value() -> (Schema, SchemaInputValueId) {
    BuildContext::build_with(|ctx, graph| {
        graph.input_value_definitions.extend([
            InputValueDefinitionRecord {
                name_id: ctx.strings.get_or_new("fieldA"),
                description_id: None,
                ty: TypeRecord {
                    definition_id: crate::DefinitionId::Object(0.into()),
                    wrapping: Wrapping::new(false),
                }, // not used
                default_value_id: None,
                directive_ids: Vec::new(),
            },
            InputValueDefinitionRecord {
                name_id: ctx.strings.get_or_new("fieldB"),
                description_id: None,
                ty: TypeRecord {
                    definition_id: crate::DefinitionId::Object(0.into()),
                    wrapping: Wrapping::new(false),
                }, // not used
                default_value_id: None,
                directive_ids: Vec::new(),
            },
        ]);
        graph.enum_value_definitions.extend([
            EnumValueRecord {
                name_id: ctx.strings.get_or_new("ACTIVE"),
                description_id: None,
                directive_ids: Default::default(),
            },
            EnumValueRecord {
                name_id: ctx.strings.get_or_new("INACTIVE"),
                description_id: None,
                directive_ids: Default::default(),
            },
        ]);
        let list = graph.input_values.push_list(vec![
            SchemaInputValueRecord::Null,
            SchemaInputValueRecord::EnumValue(EnumValueId::from(0)),
            SchemaInputValueRecord::Int(73),
        ]);
        let input_fields = graph.input_values.push_input_object(vec![
            (
                InputValueDefinitionId::from(0),
                SchemaInputValueRecord::EnumValue(EnumValueId::from(1)),
            ),
            (
                InputValueDefinitionId::from(1),
                SchemaInputValueRecord::String(ctx.strings.get_or_new("some string value")),
            ),
        ]);
        let nested_fields = graph.input_values.push_map(vec![
            (ctx.strings.get_or_new("null"), SchemaInputValueRecord::Null),
            (
                ctx.strings.get_or_new("string"),
                SchemaInputValueRecord::String(ctx.strings.get_or_new("some string value")),
            ),
            (
                ctx.strings.get_or_new("enumValue"),
                SchemaInputValueRecord::EnumValue(EnumValueId::from(0)),
            ),
            (ctx.strings.get_or_new("int"), SchemaInputValueRecord::Int(7)),
            (ctx.strings.get_or_new("bigInt"), SchemaInputValueRecord::BigInt(8)),
            (ctx.strings.get_or_new("u64"), SchemaInputValueRecord::U64(9)),
            (ctx.strings.get_or_new("float"), SchemaInputValueRecord::Float(10.0)),
            (ctx.strings.get_or_new("boolean"), SchemaInputValueRecord::Boolean(true)),
        ]);
        let fields = graph.input_values.push_map(vec![
            (
                ctx.strings.get_or_new("inputObject"),
                SchemaInputValueRecord::InputObject(input_fields),
            ),
            (ctx.strings.get_or_new("list"), SchemaInputValueRecord::List(list)),
            (
                ctx.strings.get_or_new("object"),
                SchemaInputValueRecord::Map(nested_fields),
            ),
        ]);
        graph.input_values.push_value(SchemaInputValueRecord::Map(fields))
    })
}

#[test]
fn test_display() {
    let (schema, id) = create_schema_and_input_value();
    let value = id.read(&schema);

    insta::assert_snapshot!(value, @r###"{inputObject:{fieldA:INACTIVE,fieldB:"some string value"},list:[null,ACTIVE,73],object:{null:null,string:"some string value",enumValue:ACTIVE,int:7,bigInt:8,u64:9,float:10,boolean:true}}"###);
}

#[test]
fn test_serialize() {
    let (schema, id) = create_schema_and_input_value();
    let value = id.read(&schema);

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
fn test_deserializer() {
    let (schema, id) = create_schema_and_input_value();
    let value = id.read(&schema);

    let value = serde_json::Value::deserialize(value).unwrap();

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
    let value = id.read(&schema);
    let input_value = InputValue::from(value);

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

    insta::assert_json_snapshot!(input_value, @r###"
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
    let value = id.read(&schema);

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

    let input = Input::deserialize(value).unwrap();

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

    serde::de::IgnoredAny::deserialize(value).unwrap();
}
