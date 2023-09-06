mod builder;
mod error;
pub mod graphql;
mod pagination;
mod range;
mod scalar;

use std::ops::{Not, RangeBounds};

pub use error::{BadRequestError, QueryError};
pub use pagination::{GraphqlCursor, Hit, Info, PaginatedHits, Pagination};
pub use range::Range;
pub use scalar::ScalarValue;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: Query,
    pub pagination: Pagination,
    #[serde(rename = "entity_type")]
    pub index: String,
    pub database: String,
    pub ray_id: String,
    #[serde(default)]
    pub response_parameters: QueryResponseParameters,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct QueryResponseParameters {
    pub version: QueryResponseDiscriminants,
}

type LegacyQueryResponse = PaginatedHits<String>;

// FIXME: hack of have an untagged variant (v0) for backwards compatibility.
//        to be removed once all gateway versions >= 2023-07-13.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BackwardsCompatibleQueryResponse {
    Legacy(LegacyQueryResponse),
    New(QueryResponse),
}

impl BackwardsCompatibleQueryResponse {
    pub fn into_inner(self) -> QueryResponse {
        match self {
            Self::Legacy(resp) => QueryResponse::V0(resp),
            Self::New(resp) => resp,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Default, Serialize, Deserialize))]
pub enum QueryResponse {
    #[strum_discriminants(default)]
    V0(LegacyQueryResponse), // to be removed once all gateway versions >= 2023-07-13
    V1(Result<PaginatedHits<Ulid>, QueryError>),
}

impl QueryResponse {
    pub fn normalized(self, transform_ulid: impl Fn(Ulid) -> String) -> Result<PaginatedHits<String>, QueryError> {
        match self {
            Self::V0(PaginatedHits { hits, info }) => Ok(PaginatedHits {
                hits: hits
                    .into_iter()
                    .map(|Hit { id, cursor, score }| Hit { id, cursor, score })
                    .collect(),
                info,
            }),
            Self::V1(resp) => resp.map(|PaginatedHits { hits, info }| PaginatedHits {
                hits: hits
                    .into_iter()
                    .map(|Hit { id, cursor, score }| Hit {
                        id: transform_ulid(id),
                        cursor,
                        score,
                    })
                    .collect(),
                info,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Query {
    Intersection(Vec<Query>),
    Union(Vec<Query>),
    Not(Box<Query>),
    Range { field: String, range: Range<ScalarValue> },
    In { field: String, values: Vec<ScalarValue> },
    Regex { field: String, pattern: String },
    All,
    Empty,
    IsNull { field: String },
    Text { value: String, fields: Option<Vec<String>> },
}

impl Not for Query {
    type Output = Query;

    fn not(self) -> Self::Output {
        match self {
            Query::Not(query) => *query,
            query => Query::Not(Box::new(query)),
        }
    }
}

impl Query {
    pub fn eq(field: &str, value: impl Into<ScalarValue>) -> Self {
        Query::In {
            field: field.to_string(),
            values: vec![value.into()],
        }
    }

    pub fn in_<T: Into<ScalarValue>>(field: &str, values: impl IntoIterator<Item = T>) -> Self {
        Query::In {
            field: field.to_string(),
            values: values.into_iter().map(Into::into).collect(),
        }
    }

    pub fn range<T: Clone + Into<ScalarValue>>(field: &str, range: impl RangeBounds<T>) -> Self {
        Query::Range {
            field: field.to_string(),
            range: Range::of(range),
        }
    }

    pub fn text(value: &str) -> Self {
        Query::Text {
            value: value.to_string(),
            fields: None,
        }
    }

    pub fn regex(field: &str, pattern: &str) -> Self {
        Query::Regex {
            field: field.to_string(),
            pattern: pattern.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, str::FromStr};

    use chrono::{DateTime, TimeZone, Utc};

    use super::*;

    fn ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).single().unwrap()
    }

    #[rstest::rstest]
    #[case( // 1
        r#"
        {
          "query": "All",
          "pagination": {
            "Forward": {
              "first": 100,
              "after": null
            }
          },
          "entity_type": "animal",
          "database": "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9",
          "ray_id": "7ea5c926ba983fdd"
        }
        "#,
        QueryRequest {
            query: Query::All,
            pagination: Pagination::Forward {
                first: 100,
                after: None
            },
            index: "animal".into(),
            database: "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9".into(),
            ray_id: "7ea5c926ba983fdd".into(),
            response_parameters: QueryResponseParameters {
                version: QueryResponseDiscriminants::V0
            }
        }
    )]
    #[case( // 2
        r#"
        {
          "query": {
            "Union": [
              "Empty",
              {
                "Range": {
                  "field": "a",
                  "range": {
                    "start": {
                      "Included": {
                        "Int": 0
                      }
                    },
                    "end": {
                      "Excluded": {
                        "Int": 10
                      }
                    }
                  }
                }
              },
              {
                "In": {
                  "field": "b",
                  "values": [
                    {
                      "String": "Dog"
                    }
                  ]
                }
              }
            ]
          },
          "pagination": {
            "Forward": {
              "first": 100,
              "after": "I3YT"
            }
          },
          "entity_type": "animal",
          "database": "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9",
          "ray_id": "7ea5c926ba983fdd",
          "response_parameters": {
            "version": "V0"
          }
        }
        "#,
        QueryRequest {
            query: Query::Union(vec![Query::Empty, Query::range("a", 0..10), Query::eq("b", "Dog"),]),
            pagination: Pagination::Forward {
                first: 100,
                after: Some(GraphqlCursor::from([0x23, 0x76, 0x13]))
            },
            index: "animal".into(),
            database: "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9".into(),
            ray_id: "7ea5c926ba983fdd".into(),
            response_parameters: QueryResponseParameters {
                version: QueryResponseDiscriminants::V0
            }
        }
    )]
    #[case( // 3
        r#"
        {
          "query": {
            "Intersection": [
              {
                "Not": {
                  "Text": {
                    "value": "c",
                    "fields": null
                  }
                }
              },
              {
                "Text": {
                  "value": "e",
                  "fields": [
                    "x"
                  ]
                }
              },
              {
                "Regex": {
                  "field": "d",
                  "pattern": ".*"
                }
              }
            ]
          },
          "pagination": {
            "Backward": {
              "last": 83,
              "before": "I3YT"
            }
          },
          "entity_type": "animal",
          "database": "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9",
          "ray_id": "7ea5c926ba983fdd",
          "response_parameters": {
            "version": "V1"
          }
        }
        "#,
        QueryRequest {
            query: Query::Intersection(vec![
                !Query::text("c"),
                Query::Text {
                    value: "e".into(),
                    fields: Some(vec!["x".to_string()])
                },
                Query::regex("d", ".*")
            ]),
            pagination: Pagination::Backward {
                last: 83,
                before: GraphqlCursor::from([0x23, 0x76, 0x13])
            },
            index: "animal".into(),
            database: "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9".into(),
            ray_id: "7ea5c926ba983fdd".into(),
            response_parameters: QueryResponseParameters {
                version: QueryResponseDiscriminants::V1
            }
        }
    )]
    #[case( // 4
        r#"
        {
          "query": {
            "Union": [
              {
                "In": {
                  "field": "url",
                  "values": [
                    {
                      "URL": "https://example.com"
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "email",
                  "values": [
                    {
                      "Email": "contact@example.com"
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "phone",
                  "values": [
                    {
                      "PhoneNumber": "+3300000000"
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "str",
                  "values": [
                    {
                      "String": "hi"
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "int",
                  "values": [
                    {
                      "Int": 10
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "float",
                  "values": [
                    {
                      "Float": 72.3
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "bool",
                  "values": [
                    {
                      "Boolean": false
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "ip",
                  "values": [
                    {
                      "IPAddress": "::ffff:127.0.0.1"
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "date",
                  "values": [
                    {
                      "Date": "2022-01-01"
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "datetime",
                  "values": [
                    {
                      "DateTime": 1640995200000
                    }
                  ]
                }
              },
              {
                "In": {
                  "field": "timestamp",
                  "values": [
                    {
                      "Timestamp": 1640995200000
                    }
                  ]
                }
              }
            ]
          },
          "pagination": {
            "Forward": {
              "first": 100,
              "after": null
            }
          },
          "entity_type": "animal",
          "database": "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9",
          "ray_id": "7ea5c926ba983fdd",
          "response_parameters": {
            "version": "V0"
          }
        }
        "#,
        QueryRequest {
            query: Query::Union(vec![
                Query::eq("url", ScalarValue::URL("https://example.com".to_string())),
                Query::eq("email", ScalarValue::Email("contact@example.com".to_string())),
                Query::eq("phone", ScalarValue::PhoneNumber("+3300000000".to_string())),
                Query::eq("str", "hi"),
                Query::eq("int", 10),
                Query::eq("float", 72.3),
                Query::eq("bool", false),
                Query::eq("ip", "::ffff:7f00:1".parse::<IpAddr>().unwrap()),
                Query::eq("date", ts().date_naive()),
                Query::eq("datetime", ts()),
                Query::eq("timestamp", ScalarValue::Timestamp(ts()))
            ]),
            pagination: Pagination::Forward {
                first: 100,
                after: None
            },
            index: "animal".into(),
            database: "customer-425046141993-01H4KG63J56CENC6ZC0R9NBTS9".into(),
            ray_id: "7ea5c926ba983fdd".into(),
            response_parameters: QueryResponseParameters {
                version: QueryResponseDiscriminants::V0
            }
        }
    )]
    fn request_backwards_compatbility(#[case] request: &str, #[case] expected: QueryRequest) {
        assert_eq!(serde_json::from_str::<QueryRequest>(request).unwrap(), expected);
    }

    #[rstest::rstest]
    #[case( // 1
        r#"
        {
            "hits":[
                {"id": "animal_01H5YEYMPBEMAPWDG75VK5MBBP", "cursor": "b3M2LFJBWili", "score": 7.32}
            ],
            "info":{
                "has_previous_page":false,
                "has_next_page":false,
                "total_hits":0
            }
        }
        "#,
        BackwardsCompatibleQueryResponse::Legacy(PaginatedHits {
            hits: vec![Hit {
                id: "animal_01H5YEYMPBEMAPWDG75VK5MBBP".to_string(),
                cursor: GraphqlCursor::from([0x6f, 0x73, 0x36, 0x2c, 0x52, 0x41, 0x5a, 0x29, 0x62]),
                score: 7.32,
            }],
            info: Info {
                has_previous_page: false,
                has_next_page: false,
                total_hits: 0,
            },
        })
    )]
    #[case( // 2
        r#"
        {
            "V1": {
                "Ok": {
                    "hits":[
                        {"id": "01H5YEYMPBEMAPWDG75VK5MBBP", "cursor": "b3M2LFJBWili", "score": 7.32}
                    ],
                    "info":{
                        "has_previous_page":false,
                        "has_next_page":false,
                        "total_hits":0
                    }
                }
            }
        }
        "#,
        BackwardsCompatibleQueryResponse::New(QueryResponse::V1(Ok(PaginatedHits {
            hits: vec![Hit {
                id: Ulid::from_str("01H5YEYMPBEMAPWDG75VK5MBBP").unwrap(),
                cursor: GraphqlCursor::from([0x6f, 0x73, 0x36, 0x2c, 0x52, 0x41, 0x5a, 0x29, 0x62]),
                score: 7.32,
            }],
            info: Info {
                has_previous_page: false,
                has_next_page: false,
                total_hits: 0,
            },
        })))
    )]
    #[case( // 3
        r#"
        {
            "V1": {
                "Err": "ServerError"
            }
        }
        "#,
        BackwardsCompatibleQueryResponse::New(QueryResponse::V1(Err(QueryError::ServerError)))
    )]
    #[case( // 4
        r#"
        {
            "V1": {
                "Err": {
                  "BadRequestError": {
                    "InvalidCursor": "bw=="
                  }
                }
            }
        }
        "#,
        BackwardsCompatibleQueryResponse::New(QueryResponse::V1(
            Err(QueryError::BadRequestError(BadRequestError::InvalidCursor(
                GraphqlCursor::from([0x6f]),
            )))
        ))
    )]
    #[case( // 5
        r#"
        {
            "V1": {
                "Err": {
                  "BadRequestError": {
                    "InvalidRegex": {
                      "pattern": ".*all",
                      "err": "too  complex"
                    }
                  }
                }
            }
        }
        "#,
        BackwardsCompatibleQueryResponse::New(QueryResponse::V1(
            Err(QueryError::BadRequestError(BadRequestError::InvalidRegex {
                pattern: ".*all".into(),
                err: "too  complex".into(),
            })),
        ))
    )]
    fn response_backwards_compatbility(#[case] response: &str, #[case] expected: BackwardsCompatibleQueryResponse) {
        assert_eq!(
            serde_json::from_str::<BackwardsCompatibleQueryResponse>(response).unwrap(),
            expected
        );
    }
}
