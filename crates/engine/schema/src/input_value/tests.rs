use std::borrow::Cow;

use serde::Deserialize;

use crate::{InputValue, Schema};

use super::*;

async fn create_schema_and_input_value() -> (Schema, SchemaInputValueId) {
    const SDL: &str = r###"
    input InputObject {
        fieldA: State
        fieldB: String
    }

    input ComplexObject {
        null: String
        string: String
        enumValue: State
        int: Int
        bigInt: BigInt
        float: Float
        boolean: Boolean
    }

    input All {
        inputObject: InputObject
        list: [Any]
        object: ComplexObject
    }

    enum State {
        ACTIVE
        INACTIVE
    }

    scalar Any
    scalar BigInt

    type Query {
        dummy(all: All = {
            inputObject: { fieldA: INACTIVE, fieldB: "some string value" }
            list: [null, ACTIVE, 73]
            object: {
                null: null
                string: "some string value"
                enumValue: ACTIVE
                int: 7
                bigInt: 8
                float: 10
                boolean: true
            }
        }): String
    }
    "###;

    let schema = Schema::from_sdl_or_panic(SDL).await;

    let id = schema
        .query()
        .fields()
        .find(|field| field.name() == "dummy")
        .unwrap()
        .argument_by_name("all")
        .unwrap()
        .default_value_id
        .unwrap();

    (schema, id)
}

#[tokio::test]
async fn test_display() {
    let (schema, id) = create_schema_and_input_value().await;
    let value = id.walk(&schema);

    insta::assert_snapshot!(value, @r#"{inputObject:{fieldA:INACTIVE,fieldB:"some string value"},list:[null,ACTIVE,73],object:{null:null,string:"some string value",enumValue:ACTIVE,int:7,bigInt:8,float:10,boolean:true}}"#);
}

#[tokio::test]
async fn test_serialize() {
    let (schema, id) = create_schema_and_input_value().await;
    let value = id.walk(&schema);

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
        "float": 10.0,
        "boolean": true
      }
    }
    "###);
}

#[tokio::test]
async fn test_deserializer() {
    let (schema, id) = create_schema_and_input_value().await;
    let value = id.walk(&schema);

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
        "float": 10.0,
        "boolean": true
      }
    }
    "###);
}

#[tokio::test]
async fn test_input_value() {
    let (schema, id) = create_schema_and_input_value().await;
    let value = id.walk(&schema);
    let input_value = InputValue::from(value);

    insta::assert_debug_snapshot!(input_value, @r#"
    InputObject(
        [
            (
                InputValueDefinition {
                    name: "inputObject",
                    ty: Type {
                        definition: InputObjectDefinition {
                            name: "InputObject",
                            description: None,
                            input_fields: fieldA: State, fieldB: String,
                            directives: [],
                            exists_in_subgraphs: [],
                        },
                        wrapping: Wrapping {
                            inner_is_required: false,
                            list_wrappings: [],
                            binary: "0000000000000000",
                        },
                    },
                    default_value: None,
                    directives: [],
                },
                InputObject(
                    [
                        (
                            InputValueDefinition {
                                name: "fieldA",
                                ty: Type {
                                    definition: EnumDefinition {
                                        name: "State",
                                        description: None,
                                        values: [
                                            EnumValue {
                                                name: "ACTIVE",
                                                parent_enum: "State",
                                                description: None,
                                                directives: [],
                                            },
                                            EnumValue {
                                                name: "INACTIVE",
                                                parent_enum: "State",
                                                description: None,
                                                directives: [],
                                            },
                                        ],
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            EnumValue(
                                EnumValue {
                                    name: "INACTIVE",
                                    parent_enum: "State",
                                    description: None,
                                    directives: [],
                                },
                            ),
                        ),
                        (
                            InputValueDefinition {
                                name: "fieldB",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "String",
                                        ty: String,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            String(
                                "some string value",
                            ),
                        ),
                    ],
                ),
            ),
            (
                InputValueDefinition {
                    name: "list",
                    ty: Type {
                        definition: ScalarDefinition {
                            name: "Any",
                            ty: Unknown,
                            description: None,
                            specified_by_url: None,
                            directives: [],
                            exists_in_subgraphs: [],
                        },
                        wrapping: Wrapping {
                            inner_is_required: false,
                            list_wrappings: [
                                List,
                            ],
                            binary: "0000100000000000",
                        },
                    },
                    default_value: None,
                    directives: [],
                },
                List(
                    [
                        Null,
                        UnboundEnumValue(
                            "ACTIVE",
                        ),
                        I64(
                            73,
                        ),
                    ],
                ),
            ),
            (
                InputValueDefinition {
                    name: "object",
                    ty: Type {
                        definition: InputObjectDefinition {
                            name: "ComplexObject",
                            description: None,
                            input_fields: null: String, string: String, enumValue: State, int: Int, bigInt: BigInt, float: Float, boolean: Boolean,
                            directives: [],
                            exists_in_subgraphs: [],
                        },
                        wrapping: Wrapping {
                            inner_is_required: false,
                            list_wrappings: [],
                            binary: "0000000000000000",
                        },
                    },
                    default_value: None,
                    directives: [],
                },
                InputObject(
                    [
                        (
                            InputValueDefinition {
                                name: "null",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "String",
                                        ty: String,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            Null,
                        ),
                        (
                            InputValueDefinition {
                                name: "string",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "String",
                                        ty: String,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            String(
                                "some string value",
                            ),
                        ),
                        (
                            InputValueDefinition {
                                name: "enumValue",
                                ty: Type {
                                    definition: EnumDefinition {
                                        name: "State",
                                        description: None,
                                        values: [
                                            EnumValue {
                                                name: "ACTIVE",
                                                parent_enum: "State",
                                                description: None,
                                                directives: [],
                                            },
                                            EnumValue {
                                                name: "INACTIVE",
                                                parent_enum: "State",
                                                description: None,
                                                directives: [],
                                            },
                                        ],
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            EnumValue(
                                EnumValue {
                                    name: "ACTIVE",
                                    parent_enum: "State",
                                    description: None,
                                    directives: [],
                                },
                            ),
                        ),
                        (
                            InputValueDefinition {
                                name: "int",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "Int",
                                        ty: Int,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            Int(
                                7,
                            ),
                        ),
                        (
                            InputValueDefinition {
                                name: "bigInt",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "BigInt",
                                        ty: Unknown,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            I64(
                                8,
                            ),
                        ),
                        (
                            InputValueDefinition {
                                name: "float",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "Float",
                                        ty: Float,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            Float(
                                10.0,
                            ),
                        ),
                        (
                            InputValueDefinition {
                                name: "boolean",
                                ty: Type {
                                    definition: ScalarDefinition {
                                        name: "Boolean",
                                        ty: Boolean,
                                        description: None,
                                        specified_by_url: None,
                                        directives: [],
                                        exists_in_subgraphs: [],
                                    },
                                    wrapping: Wrapping {
                                        inner_is_required: false,
                                        list_wrappings: [],
                                        binary: "0000000000000000",
                                    },
                                },
                                default_value: None,
                                directives: [],
                            },
                            Boolean(
                                true,
                            ),
                        ),
                    ],
                ),
            ),
        ],
    )
    "#);

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
        "float": 10.0,
        "boolean": true
      }
    }
    "###);
}

#[tokio::test]
async fn test_struct_deserializer() {
    let (schema, id) = create_schema_and_input_value().await;
    let value = id.walk(&schema);

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
                float: 10.0,
                boolean: true,
            },
        }
        "###);

    serde::de::IgnoredAny::deserialize(value).unwrap();
}
