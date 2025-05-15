use super::model::*;

use winnow::{
    ascii::{multispace0, multispace1},
    combinator::{delimited, dispatch, empty, eof, opt, peek, preceded, separated},
    error::{ParserError, StrContext},
    prelude::*,
    token::{any, one_of, take_while},
    Result,
};

pub fn parse(input: &str) -> std::result::Result<SelectedValue<'_>, String> {
    (ws(selected_value), eof.context(StrContext::Label("end")))
        .map(|(value, _)| value)
        .parse(input)
        .map_err(|e| e.to_string())
}

fn selected_value<'a>(input: &mut &'a str) -> Result<SelectedValue<'a>> {
    separated(
        1..,
        selected_value_entry,
        ws('|').context(StrContext::Label("value separator")),
    )
    .map(|alternatives| SelectedValue { alternatives })
    .context(StrContext::Label("value"))
    .parse_next(input)
}

/// Parses one entry in a selection, handling the different forms.
fn selected_value_entry<'a>(input: &mut &'a str) -> Result<SelectedValueEntry<'a>> {
    let parser = dispatch! { peek(any);
        '{' => selected_object_value.map(|object| SelectedValueEntry::Object { path: None, object }),
        '[' => selected_list_value.map(|list| SelectedValueEntry::List { path: None, list }),
        '.' => '.'.value(SelectedValueEntry::Identity),
        _ => selected_value_entry_with_path
    };
    parser.context(StrContext::Label("value entry")).parse_next(input)
}

fn selected_value_entry_with_path<'a>(input: &mut &'a str) -> Result<SelectedValueEntry<'a>> {
    enum Suffix<'a> {
        Object(SelectedObjectValue<'a>),
        List(SelectedListValue<'a>),
        None,
    }
    let p = path.parse_next(input)?;
    let suffix = dispatch! { peek(opt(preceded(multispace0, any)));
        Some('.') => preceded(ws('.'), selected_object_value.map(Suffix::Object)),
        Some('[') => preceded(multispace0, selected_list_value.map(Suffix::List)),
        _ => empty.value(()).map(|_| Suffix::None)
    }
    .parse_next(input)?;
    Ok(match suffix {
        Suffix::Object(object) => SelectedValueEntry::Object { path: Some(p), object },
        Suffix::List(list) => SelectedValueEntry::List { path: Some(p), list },
        Suffix::None => SelectedValueEntry::Path(p),
    })
}

/// Parses a SelectedObjectValue: { field1 field2: val ... }
fn selected_object_value<'a>(input: &mut &'a str) -> Result<SelectedObjectValue<'a>> {
    delimited(
        '{',
        ws(separated(
            0..,
            selected_object_field.context(StrContext::Label("field")),
            multispace1,
        )),
        '}',
    )
    .map(|fields| SelectedObjectValue { fields })
    .context(StrContext::Label("object"))
    .parse_next(input)
}

/// Parses a SelectedObjectField: key[: value]
fn selected_object_field<'a>(input: &mut &'a str) -> Result<SelectedObjectField<'a>> {
    let key = name.context(StrContext::Label("field name")).parse_next(input)?;
    let value = opt(
        // Optional value part: : VALUE
        preceded(ws(':'), selected_value.context(StrContext::Label("field value"))),
    )
    .parse_next(input)?;
    Ok(SelectedObjectField { key, value })
}

/// Parses a SelectedListValue: [ value ]
fn selected_list_value<'a>(input: &mut &'a str) -> Result<SelectedListValue<'a>> {
    delimited('[', ws(selected_value), ']')
        .map(SelectedListValue)
        .context(StrContext::Label("list"))
        .parse_next(input)
}

/// Parses a Path: segment1.segment2.segmentN
fn path<'a>(input: &mut &'a str) -> Result<Path<'a>> {
    (
        opt(delimited('<', ws(name), ('>', multispace0, '.')).context(StrContext::Label("type name"))),
        separated(1.., path_segment, ws('.')),
    )
        .map(|(ty, segments)| Path { ty, segments })
        .context(StrContext::Label("path"))
        .parse_next(input)
}

/// Parses a PathSegment: identifier[<type>]
fn path_segment<'s>(input: &mut &'s str) -> Result<PathSegment<'s>> {
    let (field, ty) = (
        name,
        opt(
            // Optional type constraint: < TYPENAME >
            delimited((multispace0, '<'), ws(name), '>'),
        ),
    )
        .context(StrContext::Label("path segment"))
        .parse_next(input)?;
    Ok(PathSegment { field, ty })
}

/// Parses a valid identifier (field name, type name, key)
/// Allows alphanumeric characters and underscores, must not be empty.
fn name<'s>(input: &mut &'s str) -> Result<&'s str> {
    (
        one_of(|c: char| c.is_alpha() || c == '_'),
        take_while(0.., |c: char| c.is_alphanum() || c == '_'),
    )
        .take()
        .context(StrContext::Label("name"))
        .parse_next(input)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F, O, E: ParserError<&'a str>>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_name() {
        let mut input = "fieldName";
        let result = name.parse_next(&mut input).unwrap();
        assert_eq!(result, "fieldName");
        assert_eq!(input, "");
    }

    #[test]
    fn parse_segment() {
        let mut input = "fieldName<SomeType>";
        let result = path_segment.parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            PathSegment {
                field: "fieldName",
                ty: Some("SomeType"),
            }
        );
        assert_eq!(input, "");

        let mut input = "fieldName";
        let result = path_segment.parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            PathSegment {
                field: "fieldName",
                ty: None,
            }
        );
        assert_eq!(input, "");
    }

    #[test]
    fn parse_path() {
        let mut input = "field1.field2<SomeType>.field3";
        let result = path.parse_next(&mut input).unwrap();
        assert_eq!(
            result,
            Path {
                ty: None,
                segments: vec![
                    PathSegment {
                        field: "field1",
                        ty: None,
                    },
                    PathSegment {
                        field: "field2",
                        ty: Some("SomeType"),
                    },
                    PathSegment {
                        field: "field3",
                        ty: None,
                    },
                ],
            }
        );
        assert_eq!(input, "");
    }

    #[test]
    fn test_simple_path() {
        let input = " simple_field ";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: None,
                segments: vec![PathSegment {
                    field: "simple_field",
                    ty: None,
                }],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_nested_path() {
        let input = "object.nested_field";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: None,
                segments: vec![
                    PathSegment {
                        field: "object",
                        ty: None,
                    },
                    PathSegment {
                        field: "nested_field",
                        ty: None,
                    },
                ],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_path_with_type() {
        let input = "field< MyType >";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: None,
                segments: vec![PathSegment {
                    field: "field",
                    ty: Some("MyType"),
                }],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_nested_path_with_type() {
        let input = "obj<User>.address<Addr >. street";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: None,
                segments: vec![
                    PathSegment {
                        field: "obj",
                        ty: Some("User"),
                    },
                    PathSegment {
                        field: "address",
                        ty: Some("Addr"),
                    },
                    PathSegment {
                        field: "street",
                        ty: None,
                    },
                ],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_alternative_type_conditions() {
        let input = "{ bookId: <Book>.id } | { movieId: <Movie>.id }";
        let expected = SelectedValue {
            alternatives: vec![
                SelectedValueEntry::Object {
                    path: None,
                    object: SelectedObjectValue {
                        fields: vec![SelectedObjectField {
                            key: "bookId",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: Some("Book"),
                                    segments: vec![PathSegment { field: "id", ty: None }],
                                })],
                            }),
                        }],
                    },
                },
                SelectedValueEntry::Object {
                    path: None,
                    object: SelectedObjectValue {
                        fields: vec![SelectedObjectField {
                            key: "movieId",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: Some("Movie"),
                                    segments: vec![PathSegment { field: "id", ty: None }],
                                })],
                            }),
                        }],
                    },
                },
            ],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_simple_object() {
        let input = "{ key1 key2 }";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: None,
                object: SelectedObjectValue {
                    fields: vec![
                        SelectedObjectField {
                            key: "key1",
                            value: None,
                        },
                        SelectedObjectField {
                            key: "key2",
                            value: None,
                        },
                    ],
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_object_with_values() {
        let input = "{ key1 : value1 key2 : nested.path }";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: None,
                object: SelectedObjectValue {
                    fields: vec![
                        SelectedObjectField {
                            key: "key1",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![PathSegment {
                                        field: "value1",
                                        ty: None,
                                    }],
                                })],
                            }),
                        },
                        SelectedObjectField {
                            key: "key2",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![
                                        PathSegment {
                                            field: "nested",
                                            ty: None,
                                        },
                                        PathSegment {
                                            field: "path",
                                            ty: None,
                                        },
                                    ],
                                })],
                            }),
                        },
                    ],
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_identity() {
        let input = "{ key: . }";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: None,
                object: SelectedObjectValue {
                    fields: vec![SelectedObjectField {
                        key: "key",
                        value: Some(SelectedValue {
                            alternatives: vec![SelectedValueEntry::Identity],
                        }),
                    }],
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_path_with_object() {
        let input = " data.{ field1 field2 : sub.value } ";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: Some(Path {
                    ty: None,
                    segments: vec![PathSegment {
                        field: "data",
                        ty: None,
                    }],
                }),
                object: SelectedObjectValue {
                    fields: vec![
                        SelectedObjectField {
                            key: "field1",
                            value: None,
                        },
                        SelectedObjectField {
                            key: "field2",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![
                                        PathSegment { field: "sub", ty: None },
                                        PathSegment {
                                            field: "value",
                                            ty: None,
                                        },
                                    ],
                                })],
                            }),
                        },
                    ],
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_path_with_list() {
        let input = " items [ name | id ] ";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::List {
                path: Some(Path {
                    ty: None,
                    segments: vec![PathSegment {
                        field: "items",
                        ty: None,
                    }],
                }),
                list: SelectedListValue(SelectedValue {
                    alternatives: vec![
                        SelectedValueEntry::Path(Path {
                            ty: None,
                            segments: vec![PathSegment {
                                field: "name",
                                ty: None,
                            }],
                        }),
                        SelectedValueEntry::Path(Path {
                            ty: None,
                            segments: vec![PathSegment { field: "id", ty: None }],
                        }),
                    ],
                }),
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_list_without_path() {
        let input = "[ id ]";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::List {
                path: None,
                list: SelectedListValue(SelectedValue {
                    alternatives: vec![SelectedValueEntry::Path(Path {
                        ty: None,
                        segments: vec![PathSegment { field: "id", ty: None }],
                    })],
                }),
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_alternatives() {
        let input = " path1 | path2 < Type > . sub | { key : val } | path3 [ item ]";
        let expected = SelectedValue {
            alternatives: vec![
                SelectedValueEntry::Path(Path {
                    ty: None,
                    segments: vec![PathSegment {
                        field: "path1",
                        ty: None,
                    }],
                }),
                SelectedValueEntry::Path(Path {
                    ty: None,
                    segments: vec![
                        PathSegment {
                            field: "path2",
                            ty: Some("Type"),
                        },
                        PathSegment { field: "sub", ty: None },
                    ],
                }),
                SelectedValueEntry::Object {
                    path: None,
                    object: SelectedObjectValue {
                        fields: vec![SelectedObjectField {
                            key: "key",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![PathSegment { field: "val", ty: None }],
                                })],
                            }),
                        }],
                    },
                },
                SelectedValueEntry::List {
                    path: Some(Path {
                        ty: None,
                        segments: vec![PathSegment {
                            field: "path3",
                            ty: None,
                        }],
                    }),
                    list: SelectedListValue(SelectedValue {
                        alternatives: vec![SelectedValueEntry::Path(Path {
                            ty: None,
                            segments: vec![PathSegment {
                                field: "item",
                                ty: None,
                            }],
                        })],
                    }),
                },
            ],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_whitespace_robustness() {
        // Lots of varied whitespace
        let input = " path1 < T1 > \t . \n field2 \r\n [ \t item1 \n | \r\n item2.{ subkey } \t ] \n | \t { key1 \t : \n val1 \n key2 \r\n } ";
        let expected = SelectedValue {
            alternatives: vec![
                SelectedValueEntry::List {
                    path: Some(Path {
                        ty: None,
                        segments: vec![
                            PathSegment {
                                field: "path1",
                                ty: Some("T1"),
                            },
                            PathSegment {
                                field: "field2",
                                ty: None,
                            },
                        ],
                    }),
                    list: SelectedListValue(SelectedValue {
                        alternatives: vec![
                            SelectedValueEntry::Path(Path {
                                ty: None,
                                segments: vec![PathSegment {
                                    field: "item1",
                                    ty: None,
                                }],
                            }),
                            SelectedValueEntry::Object {
                                path: Some(Path {
                                    ty: None,
                                    segments: vec![PathSegment {
                                        field: "item2",
                                        ty: None,
                                    }],
                                }),
                                object: SelectedObjectValue {
                                    fields: vec![SelectedObjectField {
                                        key: "subkey",
                                        value: None,
                                    }],
                                },
                            },
                        ],
                    }),
                },
                SelectedValueEntry::Object {
                    path: None,
                    object: SelectedObjectValue {
                        fields: vec![
                            SelectedObjectField {
                                key: "key1",
                                value: Some(SelectedValue {
                                    alternatives: vec![SelectedValueEntry::Path(Path {
                                        ty: None,
                                        segments: vec![PathSegment {
                                            field: "val1",
                                            ty: None,
                                        }],
                                    })],
                                }),
                            },
                            SelectedObjectField {
                                key: "key2",
                                value: None,
                            },
                        ],
                    },
                },
            ],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("Whitespace test failed: {}", e),
        }
    }

    #[test]
    fn test_empty_object() {
        let input = "{}";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: None,
                object: SelectedObjectValue {
                    fields: vec![], // Empty fields
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }

        let input = "path.{}";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: Some(Path {
                    ty: None,
                    segments: vec![PathSegment {
                        field: "path",
                        ty: None,
                    }],
                }),
                object: SelectedObjectValue { fields: vec![] },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_nested_complex() {
        // Object within list within object alternative
        let input = " a | b.{ c: d [ e | { f : g.h<T> } ] } ";
        let expected = SelectedValue {
            alternatives: vec![
                SelectedValueEntry::Path(Path {
                    ty: None,
                    segments: vec![PathSegment { field: "a", ty: None }],
                }),
                SelectedValueEntry::Object {
                    path: Some(Path {
                        ty: None,
                        segments: vec![PathSegment { field: "b", ty: None }],
                    }),
                    object: SelectedObjectValue {
                        fields: vec![SelectedObjectField {
                            key: "c",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::List {
                                    path: Some(Path {
                                        ty: None,
                                        segments: vec![PathSegment { field: "d", ty: None }],
                                    }),
                                    list: SelectedListValue(SelectedValue {
                                        alternatives: vec![
                                            SelectedValueEntry::Path(Path {
                                                ty: None,
                                                segments: vec![PathSegment { field: "e", ty: None }],
                                            }),
                                            SelectedValueEntry::Object {
                                                path: None,
                                                object: SelectedObjectValue {
                                                    fields: vec![SelectedObjectField {
                                                        key: "f",
                                                        value: Some(SelectedValue {
                                                            alternatives: vec![SelectedValueEntry::Path(Path {
                                                                ty: None,
                                                                segments: vec![
                                                                    PathSegment { field: "g", ty: None },
                                                                    PathSegment {
                                                                        field: "h",
                                                                        ty: Some("T"),
                                                                    },
                                                                ],
                                                            })],
                                                        }),
                                                    }],
                                                },
                                            },
                                        ],
                                    }),
                                }],
                            }),
                        }],
                    },
                },
            ],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_no_space_around_colon() {
        let input = "{key1:value1 key2:nested.path}"; // No spaces around ':'
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: None,
                object: SelectedObjectValue {
                    fields: vec![
                        SelectedObjectField {
                            key: "key1",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![PathSegment {
                                        field: "value1",
                                        ty: None,
                                    }],
                                })],
                            }),
                        },
                        SelectedObjectField {
                            key: "key2",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![
                                        PathSegment {
                                            field: "nested",
                                            ty: None,
                                        },
                                        PathSegment {
                                            field: "path",
                                            ty: None,
                                        },
                                    ],
                                })],
                            }),
                        },
                    ],
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_underscore_names() {
        let input = "_leading._deep_._member";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: None,
                segments: vec![
                    PathSegment {
                        field: "_leading",
                        ty: None,
                    },
                    PathSegment {
                        field: "_deep_",
                        ty: None,
                    },
                    PathSegment {
                        field: "_member",
                        ty: None,
                    },
                ],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }

        let input = "_"; // Single underscore
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: None,
                segments: vec![PathSegment { field: "_", ty: None }],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_duplicate_object_keys() {
        // The parser should technically allow this, as the model uses Vec<SelectedObjectField>
        // Further semantic checks might disallow duplicates, but the parser itself shouldn't fail.
        let input = "{ key key : value }";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Object {
                path: None,
                object: SelectedObjectValue {
                    fields: vec![
                        SelectedObjectField {
                            key: "key",
                            value: None,
                        },
                        SelectedObjectField {
                            key: "key",
                            value: Some(SelectedValue {
                                alternatives: vec![SelectedValueEntry::Path(Path {
                                    ty: None,
                                    segments: vec![PathSegment {
                                        field: "value",
                                        ty: None,
                                    }],
                                })],
                            }),
                        },
                    ],
                },
            }],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_empty_input() {
        insta::assert_snapshot!(parse("").unwrap_err(), @r#"
        ^
        invalid value entry
        "#);
    }

    #[test]
    fn test_only_whitespace() {
        insta::assert_snapshot!(parse("   \t\n ").unwrap_err(), @r#"
        parse error at line 2, column 2
          |
        2 |  
          |  ^
        invalid value entry
        "#);
    }

    #[test]
    fn test_invalid_syntax() {
        // Incomplete path
        insta::assert_snapshot!(parse("path.").unwrap_err(), @r#"
        path.
             ^
        invalid object
        "#);

        // Incomplete alternative
        insta::assert_snapshot!(parse("path.|").unwrap_err(), @r#"
        path.|
             ^
        invalid object
        "#);

        // Unclosed object
        insta::assert_snapshot!(parse("{ key ").unwrap_err(), @r#"
        { key 
              ^
        invalid object
        "#);

        // Unclosed list
        insta::assert_snapshot!(parse("[ val").unwrap_err(), @r#"
        [ val
             ^
        invalid list
        "#);

        // Unclosed type constraint
        insta::assert_snapshot!(parse("path<Type").unwrap_err(), @r#"
        path<Type
            ^
        invalid end
        "#);

        // Missing value after colon
        insta::assert_snapshot!(parse("key :").unwrap_err(), @r#"
        key :
            ^
        invalid end
        "#);

        // Empty alternative - separated requires non-empty elements
        insta::assert_snapshot!(parse("a | | b").unwrap_err(), @r#"
        a | | b
          ^
        invalid end
        "#);
    }

    #[test]
    fn test_trailing_input() {
        // parse ensures all input is consumed
        insta::assert_snapshot!(parse("field extra").unwrap_err(), @r#"
        field extra
              ^
        invalid end
        "#);

        insta::assert_snapshot!(parse("{key} trailing").unwrap_err(), @r#"
        {key} trailing
              ^
        invalid end
        "#);
    }

    #[test]
    fn test_invalid_characters_in_name() {
        insta::assert_snapshot!(parse("invalid-name").unwrap_err(), @r#"
        invalid-name
               ^
        invalid end
        "#);
        insta::assert_snapshot!(parse("obj.invalid-segment").unwrap_err(), @r#"
        obj.invalid-segment
                   ^
        invalid end
        "#);
        insta::assert_snapshot!(parse("{ invalid-key }").unwrap_err(), @r#"
        { invalid-key }
                 ^
        invalid object
        "#);
        insta::assert_snapshot!(parse("path<invalid-type!>").unwrap_err(), @r#"
        path<invalid-type!>
            ^
        invalid end
        "#);
    }

    #[test]
    fn test_invalid_start_characters() {
        // Cannot start path segment with number
        insta::assert_snapshot!(parse("1path").unwrap_err(), @r#"
        1path
        ^
        invalid name
        "#);
        // Cannot start object key with number
        insta::assert_snapshot!(parse("{ 1key }").unwrap_err(), @r#"
        { 1key }
          ^
        invalid object
        "#);
        // Dot cannot start input
        insta::assert_snapshot!(parse(".path").unwrap_err(), @r#"
        .path
         ^
        invalid end
        "#);
        // Pipe cannot start input
        insta::assert_snapshot!(parse("| path").unwrap_err(), @r#"
        | path
        ^
        invalid name
        "#);
        // Colon cannot start input
        insta::assert_snapshot!(parse(": value").unwrap_err(), @r#"
        : value
        ^
        invalid name
        "#);
    }

    #[test]
    fn test_invalid_structure() {
        // Missing key before colon
        insta::assert_snapshot!(parse("{ : value }").unwrap_err(), @r#"
        { : value }
          ^
        invalid object
        "#);

        // Dot after object
        insta::assert_snapshot!(parse("{ key }.extra").unwrap_err(), @r#"
        { key }.extra
               ^
        invalid end
        "#); // Error might point after '}' as it expects EOF

        // List after object
        insta::assert_snapshot!(parse("{ key }[extra]").unwrap_err(), @r#"
        { key }[extra]
               ^
        invalid end
        "#); // Error might point after '}'

        // Path segment after list
        insta::assert_snapshot!(parse("path[item].extra").unwrap_err(), @r#"
        path[item].extra
                  ^
        invalid end
        "#); // Error might point after ']'

        // Double dot in path
        insta::assert_snapshot!(parse("path..segment").unwrap_err(), @r#"
        path..segment
             ^
        invalid object
        "#);

        // Double pipe
        insta::assert_snapshot!(parse("path || segment").unwrap_err(), @r#"
        path || segment
             ^
        invalid end
        "#); // Already covered by 'a | | b' but good to have variation

        // Empty list contents - selected_value requires at least one entry
        insta::assert_snapshot!(parse("path[]").unwrap_err(), @r#"
        path[]
             ^
        invalid name
        "#);
        insta::assert_snapshot!(parse("path[ ]").unwrap_err(), @r#"
        path[ ]
              ^
        invalid name
        "#);

        // Empty type constraint
        insta::assert_snapshot!(parse("path<>").unwrap_err(), @r#"
        path<>
            ^
        invalid end
        "#);
        insta::assert_snapshot!(parse("path< >").unwrap_err(), @r#"
        path< >
            ^
        invalid end
        "#);

        // Object suffix directly after path type constraint (needs space/end)
        insta::assert_snapshot!(parse("path<T>{key}").unwrap_err(), @r#"
        path<T>{key}
               ^
        invalid end
        "#);
    }

    #[test]
    fn test_trailing_separators() {
        insta::assert_snapshot!(parse("path.").unwrap_err(), @r#"
        path.
             ^
        invalid object
        "#); // Slightly different context than previous test

        insta::assert_snapshot!(parse("path |").unwrap_err(), @r#"
        path |
             ^
        invalid end
        "#); // Slightly different context

        insta::assert_snapshot!(parse("{ key : }").unwrap_err(), @r#"
        { key : }
              ^
        invalid object
        "#); // Slightly different context
    }

    #[test]
    fn test_round_trip_formatting() {
        let inputs = vec![
            ("simple", "simple"),
            (" a . b < T > ", "a.b<T>"), // Note canonical spacing
            ("{ k1 k2: v }", "{ k1 k2: v }"),
            (" p . { f1 f2 : s } ", "p.{ f1 f2: s }"),
            ("[ i1  ] ", "[i1]"),
            (" l [ i1 | i2 < U > ] ", "l[i1 | i2<U>]"),
            ("a | b.{c} | d[e]", "a | b.{ c } | d[e]"), // Slight format variation for object
            (
                "complex<T1>.path[ item | { key : val<T2>.sub } ] | another",
                "complex<T1>.path[item | { key: val<T2>.sub }] | another",
            ),
            ("{ key : . }", "{ key: . }"),
        ];

        for (original, canonical) in inputs {
            match parse(original) {
                Ok(parsed) => {
                    assert_eq!(parsed.to_string(), canonical, "Round trip failed for: {}", original);
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_path_with_start_type() {
        let input = "<Type>.field1.field2";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: Some("Type"),
                segments: vec![
                    PathSegment {
                        field: "field1",
                        ty: None,
                    },
                    PathSegment {
                        field: "field2",
                        ty: None,
                    },
                ],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn test_path_with_start_type_and_segment_type() {
        let input = "<Type>.field1<InnerType>.field2";
        let expected = SelectedValue {
            alternatives: vec![SelectedValueEntry::Path(Path {
                ty: Some("Type"),
                segments: vec![
                    PathSegment {
                        field: "field1",
                        ty: Some("InnerType"),
                    },
                    PathSegment {
                        field: "field2",
                        ty: None,
                    },
                ],
            })],
        };
        match parse(input) {
            Ok(result) => assert_eq!(result, expected),
            Err(e) => panic!("{}", e),
        }
    }
}
