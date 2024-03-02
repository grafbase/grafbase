mod cache_key;
mod de;
mod extensions;
mod query;

pub use cache_key::*;
pub use extensions::*;
use id_newtypes::IdRange;
pub use query::*;
use serde::{de::DeserializeOwned, Deserialize};
use std::borrow::Cow;

pub enum HttpGraphqlRequest<'a> {
    /// Graphql request is in the query string, for GET http requests
    Query(Cow<'a, str>),
    /// Graphql request in the request body (json), for POST http requests
    JsonBody(Cow<'a, [u8]>),
    JsonBodyBytes(bytes::Bytes),
}

impl HttpGraphqlRequest<'_> {
    pub fn as_ref(&self) -> HttpGraphqlRequest<'_> {
        match self {
            HttpGraphqlRequest::Query(s) => HttpGraphqlRequest::Query(s.as_ref().into()),
            HttpGraphqlRequest::JsonBody(b) => HttpGraphqlRequest::JsonBody(b.as_ref().into()),
            HttpGraphqlRequest::JsonBodyBytes(b) => HttpGraphqlRequest::JsonBodyBytes(b.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged, bound = "E: Default + Deserialize<'de>")]
pub enum BatchGraphqlRequest<'a, E> {
    #[serde(borrow)]
    Single(GraphqlRequest<'a, E>),
    #[serde(borrow, deserialize_with = "self::de::deserialize_non_empty_vec")]
    Batch(Vec<GraphqlRequest<'a, E>>),
}

impl<'a, E> BatchGraphqlRequest<'a, E> {
    pub fn from_http_request(request: &'a HttpGraphqlRequest<'a>) -> Result<Self, String>
    where
        E: Default + DeserializeOwned,
    {
        match request {
            HttpGraphqlRequest::Query(s) => Self::from_query(s.as_ref()).map_err(|err| err.to_string()),
            HttpGraphqlRequest::JsonBody(b) => Self::from_json(b.as_ref()).map_err(|err| err.to_string()),
            HttpGraphqlRequest::JsonBodyBytes(b) => Self::from_json(b.as_ref()).map_err(|err| err.to_string()),
        }
    }

    pub fn from_json(bytes: &'a [u8]) -> serde_json::Result<Self>
    where
        E: Default + Deserialize<'a>,
    {
        serde_json::from_slice(bytes)
    }

    pub fn from_query(query: &'a str) -> Result<Self, serde_urlencoded::de::Error>
    where
        E: Default + DeserializeOwned,
    {
        let request: BorrowedQueryRequest<'a, E> = serde_urlencoded::from_str(query)?;
        Ok(request.into())
    }
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(bound = "E: Default + Deserialize<'de>")]
pub struct GraphqlRequest<'a, E> {
    #[serde(default, borrow)]
    pub operation_name: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub doc_id: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub query: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub variables: BorrowedVariables<'a>,
    #[serde(default)]
    pub extensions: E,
}

impl<'a, E> From<GraphqlRequest<'a, E>> for BatchGraphqlRequest<'a, E> {
    fn from(value: GraphqlRequest<'a, E>) -> Self {
        Self::Single(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct BorrowedVariables<'a> {
    pub root: BorrowedValue<'a>,
    values: Vec<BorrowedValue<'a>>,
    key_values: Vec<(Cow<'a, str>, BorrowedValue<'a>)>,
}

id_newtypes::U32! {
    BorrowedVariables<'a>.values[RequestValueId] => BorrowedValue<'a> | unless "Too many request values",
    BorrowedVariables<'a>.key_values[RequestKeyValueId] => (Cow<'a, str>, BorrowedValue<'a>) | unless "Too many request key values",
}

impl BorrowedVariables<'_> {
    pub fn into_owned(self) -> BorrowedVariables<'static> {
        BorrowedVariables {
            root: self.root.into_owned(),
            values: self.values.into_iter().map(BorrowedValue::into_owned).collect(),
            key_values: self
                .key_values
                .into_iter()
                .map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_owned()))
                .collect(),
        }
    }
}

impl Default for BorrowedVariables<'_> {
    fn default() -> Self {
        Self {
            root: BorrowedValue::Null,
            values: Default::default(),
            key_values: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BorrowedValue<'a> {
    Null,
    Bool(bool),
    F64(f64),
    I64(i64),
    U64(u64),
    String(Cow<'a, str>),
    List(IdRange<RequestValueId>),
    // Sorted by the key
    Map(IdRange<RequestKeyValueId>),
}

impl BorrowedValue<'_> {
    pub(crate) fn stable_id(&self) -> u8 {
        match self {
            BorrowedValue::Null => 0,
            BorrowedValue::Bool(_) => 1,
            BorrowedValue::F64(_) => 2,
            BorrowedValue::I64(_) => 3,
            BorrowedValue::U64(_) => 4,
            BorrowedValue::String(_) => 5,
            BorrowedValue::List(_) => 6,
            BorrowedValue::Map(_) => 7,
        }
    }

    fn into_owned(self) -> BorrowedValue<'static> {
        match self {
            BorrowedValue::String(s) => BorrowedValue::String(s.into_owned().into()),
            BorrowedValue::Null => BorrowedValue::Null,
            BorrowedValue::Bool(b) => BorrowedValue::Bool(b),
            BorrowedValue::F64(f) => BorrowedValue::F64(f),
            BorrowedValue::I64(i) => BorrowedValue::I64(i),
            BorrowedValue::U64(u) => BorrowedValue::U64(u),
            BorrowedValue::List(r) => BorrowedValue::List(r),
            BorrowedValue::Map(r) => BorrowedValue::Map(r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[test]
    fn size_check() {
        assert_eq!(std::mem::size_of::<BorrowedValue<'static>>(), 24);
    }

    #[test]
    fn serde() {
        let data = json!({
            "query": "query { a }",
            "variables": {
                "a": 1,
                "b": [1, -2, 3],
                "c": {
                    "d": "e"
                }
            }
        });

        let request = GraphqlRequest::<'_, serde::de::IgnoredAny>::deserialize(&data).unwrap();
        insta::assert_debug_snapshot!(request, @r###"
        GraphqlRequest {
            operation_name: None,
            doc_id: None,
            query: Some(
                "query { a }",
            ),
            variables: BorrowedVariables {
                root: Map(
                    IdRange {
                        start: 1,
                        end: 4,
                    },
                ),
                values: [
                    U64(
                        1,
                    ),
                    I64(
                        -2,
                    ),
                    U64(
                        3,
                    ),
                ],
                key_values: [
                    (
                        "d",
                        String(
                            "e",
                        ),
                    ),
                    (
                        "a",
                        U64(
                            1,
                        ),
                    ),
                    (
                        "b",
                        List(
                            IdRange {
                                start: 0,
                                end: 3,
                            },
                        ),
                    ),
                    (
                        "c",
                        Map(
                            IdRange {
                                start: 0,
                                end: 1,
                            },
                        ),
                    ),
                ],
            },
            extensions: IgnoredAny,
        }
        "###);

        let bytes = serde_json::to_vec(&data).unwrap();
        let request: GraphqlRequest<'_, serde::de::IgnoredAny> = serde_json::from_slice(&bytes[..]).unwrap();
        insta::assert_debug_snapshot!(request, @r###"
        GraphqlRequest {
            operation_name: None,
            doc_id: None,
            query: Some(
                "query { a }",
            ),
            variables: BorrowedVariables {
                root: Map(
                    IdRange {
                        start: 1,
                        end: 4,
                    },
                ),
                values: [
                    U64(
                        1,
                    ),
                    I64(
                        -2,
                    ),
                    U64(
                        3,
                    ),
                ],
                key_values: [
                    (
                        "d",
                        String(
                            "e",
                        ),
                    ),
                    (
                        "a",
                        U64(
                            1,
                        ),
                    ),
                    (
                        "b",
                        List(
                            IdRange {
                                start: 0,
                                end: 3,
                            },
                        ),
                    ),
                    (
                        "c",
                        Map(
                            IdRange {
                                start: 0,
                                end: 1,
                            },
                        ),
                    ),
                ],
            },
            extensions: IgnoredAny,
        }
        "###);
    }

    #[test]
    fn serde_batch() {
        let req1 = json!({
            "query": "query { a }",
            "variables": {
                "a": 1,
                "b": [1, -2, 3],
                "c": {
                    "d": "e"
                }
            }
        });
        let req2 = json!({
            "query": "mutation { doStuff }",
            "variables": {
                "a": ["doggy"],
            }
        });

        let request = BatchGraphqlRequest::<'_, serde::de::IgnoredAny>::deserialize(&req1).unwrap();
        insta::assert_debug_snapshot!(request, @r###"
        Single(
            GraphqlRequest {
                operation_name: None,
                doc_id: None,
                query: Some(
                    "query { a }",
                ),
                variables: BorrowedVariables {
                    root: Map(
                        IdRange {
                            start: 1,
                            end: 4,
                        },
                    ),
                    values: [
                        U64(
                            1,
                        ),
                        I64(
                            -2,
                        ),
                        U64(
                            3,
                        ),
                    ],
                    key_values: [
                        (
                            "d",
                            String(
                                "e",
                            ),
                        ),
                        (
                            "a",
                            U64(
                                1,
                            ),
                        ),
                        (
                            "b",
                            List(
                                IdRange {
                                    start: 0,
                                    end: 3,
                                },
                            ),
                        ),
                        (
                            "c",
                            Map(
                                IdRange {
                                    start: 0,
                                    end: 1,
                                },
                            ),
                        ),
                    ],
                },
                extensions: IgnoredAny,
            },
        )
        "###);

        let bytes = serde_json::to_vec(&vec![req1, req2]).unwrap();
        let request: BatchGraphqlRequest<'_, serde::de::IgnoredAny> = serde_json::from_slice(&bytes[..]).unwrap();
        insta::assert_debug_snapshot!(request, @r###"
        Batch(
            [
                GraphqlRequest {
                    operation_name: None,
                    doc_id: None,
                    query: Some(
                        "query { a }",
                    ),
                    variables: BorrowedVariables {
                        root: Map(
                            IdRange {
                                start: 1,
                                end: 4,
                            },
                        ),
                        values: [
                            U64(
                                1,
                            ),
                            I64(
                                -2,
                            ),
                            U64(
                                3,
                            ),
                        ],
                        key_values: [
                            (
                                "d",
                                String(
                                    "e",
                                ),
                            ),
                            (
                                "a",
                                U64(
                                    1,
                                ),
                            ),
                            (
                                "b",
                                List(
                                    IdRange {
                                        start: 0,
                                        end: 3,
                                    },
                                ),
                            ),
                            (
                                "c",
                                Map(
                                    IdRange {
                                        start: 0,
                                        end: 1,
                                    },
                                ),
                            ),
                        ],
                    },
                    extensions: IgnoredAny,
                },
                GraphqlRequest {
                    operation_name: None,
                    doc_id: None,
                    query: Some(
                        "mutation { doStuff }",
                    ),
                    variables: BorrowedVariables {
                        root: Map(
                            IdRange {
                                start: 0,
                                end: 1,
                            },
                        ),
                        values: [
                            String(
                                "doggy",
                            ),
                        ],
                        key_values: [
                            (
                                "a",
                                List(
                                    IdRange {
                                        start: 0,
                                        end: 1,
                                    },
                                ),
                            ),
                        ],
                    },
                    extensions: IgnoredAny,
                },
            ],
        )
        "###);
    }
}
