use std::{iter, ops::Bound};

use serde::{Deserialize, Serialize};

use super::{
    builder::{IntersectionQueryBuilder, UnionQueryBuilder},
    BadRequestError, Query, Range, ScalarValue,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GraphqlQuery {
    pub text: Option<String>,
    pub fields: Option<Vec<String>>,
    pub filter: Option<Filter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Filter {
    Scalar { field: String, condition: ScalarCondition },
    List { field: String, condition: ListCondition },
    All(Vec<Filter>),
    Any(Vec<Filter>),
    None(Vec<Filter>),
    Not(Box<Filter>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ScalarCondition {
    Lt(ScalarValue),
    Lte(ScalarValue),
    Eq(ScalarValue),
    Gte(ScalarValue),
    Gt(ScalarValue),
    In(Vec<ScalarValue>),
    NotIn(Vec<ScalarValue>),
    Neq(ScalarValue),
    IsNull(bool),
    Regex { pattern: String },
    // Allows more complex conditions for Lists
    All(Vec<ScalarCondition>),
    Any(Vec<ScalarCondition>),
    None_(Vec<ScalarCondition>),
    Not(Box<ScalarCondition>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ListCondition {
    HasAny(ScalarCondition),
    HasNone(ScalarCondition),
    IsEmpty(bool),
}

impl TryFrom<GraphqlQuery> for Query {
    type Error = BadRequestError;

    // FIXME: The transformation from GraphqlQuery to Query is done in the worker to avoid having
    //        to copy-paste all of that code into the cli repository... With OS gateway, Query
    //        shouldn't be exposed anymore.
    fn try_from(value: GraphqlQuery) -> Result<Self, Self::Error> {
        let GraphqlQuery { text, fields, filter } = value;
        IntersectionQueryBuilder::build_from(
            vec![
                text.map(|text| Ok(Query::Text { value: text, fields })),
                filter.map(Query::try_from),
            ]
            .into_iter()
            .flatten(),
        )
    }
}

// Looks a bit silly but lets us use IntersectionQueryBuilder::build_from() on results directly.
impl TryFrom<Result<Query, BadRequestError>> for Query {
    type Error = BadRequestError;

    fn try_from(value: Result<Query, BadRequestError>) -> Result<Self, Self::Error> {
        value
    }
}

impl TryFrom<Filter> for Query {
    type Error = BadRequestError;

    fn try_from(filter: Filter) -> Result<Self, Self::Error> {
        use Filter::*;
        use ListCondition::*;

        match filter {
            All(filters) => IntersectionQueryBuilder::build_from(filters),
            Any(filters) => UnionQueryBuilder::build_from(filters),
            None(filters) => UnionQueryBuilder::build_from(filters).map(|query| !query),
            Not(filter) => Query::try_from(*filter).map(|query| !query),
            Scalar { field, condition } => Query::try_from((field, condition)),
            List { field, condition } => match condition {
                // Tantivy stores all fields as lists. If any of the item (term) matches, the
                // document matches. Hence HasAny & HasNone is just equivalent to treating the
                // field as a single scalar. But we there's no out of the box support for HasAll AFAIK currently.
                HasAny(conditions) => Query::try_from((field, conditions)),
                HasNone(conditions) => Query::try_from((field, conditions)).map(|query| !query),
                // Being empty means no value exists for the field, so it's null.
                IsEmpty(value) => Query::try_from((field, ScalarCondition::IsNull(value))),
            },
        }
    }
}

impl TryFrom<(String, ScalarCondition)> for Query {
    type Error = BadRequestError;

    fn try_from((field, condition): (String, ScalarCondition)) -> Result<Self, Self::Error> {
        use ScalarCondition::*;

        Ok(match condition {
            Lt(value) => Query::Range {
                field,
                range: Range::of(..value),
            },
            Lte(value) => Query::Range {
                field,
                range: Range::of(..=value),
            },
            Eq(value) => Query::In {
                field,
                values: vec![value],
            },
            Gte(value) => Query::Range {
                field,
                range: Range::of(value..),
            },
            Gt(value) => Query::Range {
                field,
                range: Range {
                    start: Bound::Excluded(value),
                    end: Bound::Unbounded,
                },
            },
            In(values) => Query::In { field, values },
            NotIn(values) => !Query::In { field, values },
            Neq(value) => !Query::In {
                field,
                values: vec![value],
            },
            IsNull(true) => Query::IsNull { field },
            IsNull(false) => !Query::IsNull { field },
            All(conditions) => IntersectionQueryBuilder::build_from(iter::repeat(field).zip(conditions))?,
            Any(conditions) => UnionQueryBuilder::build_from(iter::repeat(field).zip(conditions))?,
            None_(conditions) => !UnionQueryBuilder::build_from(iter::repeat(field).zip(conditions))?,
            Not(condition) => Query::try_from((field, *condition)).map(|query| match query {
                Query::Not(query) => *query,
                query => !query,
            })?,
            Regex { pattern } => Query::Regex { field, pattern },
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::{NaiveDate, TimeZone, Utc};

    use super::*;

    macro_rules! vec_into {
        ($($x:expr),+ $(,)?) => {
            vec![$($x.into(),)+]
        };
    }

    fn filter(field: &str, condition: ScalarCondition) -> Filter {
        Filter::Scalar {
            field: field.to_string(),
            condition,
        }
    }

    fn lt<T: Into<ScalarValue>>(value: T) -> ScalarCondition {
        ScalarCondition::Lt(value.into())
    }

    fn gt<T: Into<ScalarValue>>(value: T) -> ScalarCondition {
        ScalarCondition::Gt(value.into())
    }

    fn lte<T: Into<ScalarValue>>(value: T) -> ScalarCondition {
        ScalarCondition::Lte(value.into())
    }

    fn gte<T: Into<ScalarValue>>(value: T) -> ScalarCondition {
        ScalarCondition::Gte(value.into())
    }

    fn ip4(ip: u32) -> IpAddr {
        IpAddr::V4(Ipv4Addr::from(ip))
    }

    #[rstest::rstest]
    #[case( // 1
        filter("x", ScalarCondition::Eq(ScalarValue::Boolean(false))),
        Query::In {
            field: "x".into(),
            values: vec![ScalarValue::Boolean(false)]
        }
    )]
    #[case( // 2
        filter("x", ScalarCondition::Neq(ScalarValue::Boolean(false))),
        Query::Not(Box::new(Query::In {
            field: "x".into(),
            values: vec![ScalarValue::Boolean(false)]
        }))
    )]
    #[case( // 3
        filter("x", lte(10)),
        Query::Range { field: "x".to_string(), range: Range::of(..=10) }
    )]
    #[case( // 4
        filter("x", ScalarCondition::In(vec_into![1, 5, 7])),
        Query::In {
            field: "x".into(),
            values: vec_into![1, 5, 7]
        }
    )]
    #[case( // 5
        filter("x", ScalarCondition::NotIn(vec_into![1, 5, 7])),
        Query::Not(Box::new(Query::In {
            field: "x".into(),
            values: vec_into![1, 5, 7]
        }))
    )]
    #[case( // 6
        Filter::Not(Box::new(filter("x", ScalarCondition::NotIn(vec_into![1, 5, 7])))),
        Query::In {
            field: "x".into(),
            values: vec_into![1, 5, 7]
        }
    )]
    fn test_basic(#[case] filter: Filter, #[case] query: Query) {
        assert_eq!(Query::try_from(filter).unwrap(), query);
    }

    #[rstest::rstest]
    #[case( // 1
        vec![gt(10), gt(17), lt(91), lt(100)],
        Range::of((Bound::Excluded(17), Bound::Excluded(91)))
    )]
    #[case( // 2
        vec![gt(3), gte(10), lte(90), lt(100)],
        Range::of(10..=90)
    )]
    #[case( // 3
        vec![gt(10), gte(10), lte(100), lt(100)],
        Range::of((Bound::Excluded(10), Bound::Excluded(100)))
    )]
    #[case( // 4
        vec![gte(10), gt(10)],
        Range::of((Bound::Excluded(10), Bound::Unbounded))
    )]
    #[case( // 5
        vec![lte(100), lt(100)],
        Range::of((Bound::Unbounded, Bound::Excluded(100)))
    )]
    #[case( // 6 String
        vec![gt("a"), gte("b"), lte("r"), lt("s")],
        Range::of("b"..="r")
    )]
    #[case( // 7 Float
        vec![gt(3.1), gte(10.2), lte(90.3), lt(100.4)],
        Range::of(10.2..=90.3)
    )]
    #[case( // 8 IpAddr
        vec![gt(ip4(1u32)), gte(ip4(2u32)), lte(ip4(10u32)), lt(ip4(11u32))],
        Range::of(ip4(2u32)..=ip4(10))
    )]
    #[case( // 9 Date
        vec![
            gt(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap()),
            gte(NaiveDate::from_ymd_opt(2022, 2, 1).unwrap()),
            lte(NaiveDate::from_ymd_opt(2022, 9, 1).unwrap()),
            lt(NaiveDate::from_ymd_opt(2022, 10, 1).unwrap())
        ],
        Range::of((
            Bound::Included(NaiveDate::from_ymd_opt(2022, 2, 1).unwrap()),
            Bound::Included(NaiveDate::from_ymd_opt(2022, 9, 1).unwrap())
        ))
    )]
    #[case( // 10 DateTime
        vec![
            gt(Utc.timestamp_nanos(1_431_648_000_000_000)),
            gte(Utc.timestamp_nanos(1_431_648_000_000_002)),
            lte(Utc.timestamp_nanos(1_431_648_000_000_010)),
            lt(Utc.timestamp_nanos(1_431_648_000_000_012)),
        ],
        Range::of((
            Bound::Included(Utc.timestamp_nanos(1_431_648_000_000_002)),
            Bound::Included(Utc.timestamp_nanos(1_431_648_000_000_010))
        ))
    )]
    #[case( // 11 Timestamp
        vec![
            gt(ScalarValue::Timestamp(Utc.timestamp_nanos( 1_431_648_000_000_000))),
            gte(ScalarValue::Timestamp(Utc.timestamp_nanos(1_431_648_000_000_002))),
            lte(ScalarValue::Timestamp(Utc.timestamp_nanos(1_431_648_000_000_010))),
            lt(ScalarValue::Timestamp(Utc.timestamp_nanos(1_431_648_000_000_012))),
        ],
        Range::of((
            Bound::Included(ScalarValue::Timestamp(Utc.timestamp_nanos(1_431_648_000_000_002))),
            Bound::Included(ScalarValue::Timestamp(Utc.timestamp_nanos(1_431_648_000_000_010)))
        ))
    )]
    #[case( // 12 URL
        vec![
            gt(ScalarValue::URL("a".to_string())),
            gte(ScalarValue::URL("b".to_string())),
            lte(ScalarValue::URL("r".to_string())),
            lt(ScalarValue::URL("s".to_string())),
        ],
        Range::of((
            Bound::Included(ScalarValue::URL("b".to_string())),
            Bound::Included(ScalarValue::URL("r".to_string()))
        ))
    )]
    #[case( // 13 Email
        vec![
            gt(ScalarValue::Email("a".to_string())),
            gte(ScalarValue::Email("b".to_string())),
            lte(ScalarValue::Email("r".to_string())),
            lt(ScalarValue::Email("s".to_string())),
        ],
        Range::of((
            Bound::Included(ScalarValue::Email("b".to_string())),
            Bound::Included(ScalarValue::Email("r".to_string()))
        ))
    )]
    #[case( // 14 PhoneNumber
        vec![
            gt(ScalarValue::PhoneNumber("a".to_string())),
            gte(ScalarValue::PhoneNumber("b".to_string())),
            lte(ScalarValue::PhoneNumber("r".to_string())),
            lt(ScalarValue::PhoneNumber("s".to_string())),
        ],
        Range::of((
            Bound::Included(ScalarValue::PhoneNumber("b".to_string())),
            Bound::Included(ScalarValue::PhoneNumber("r".to_string()))
        ))
    )]
    fn test_ranges(#[case] conditions: Vec<ScalarCondition>, #[case] range: Range<ScalarValue>) {
        assert_eq!(
            Query::try_from(Filter::All(
                conditions.into_iter().map(|cond| filter("x", cond)).collect()
            ))
            .unwrap(),
            Query::Range {
                field: "x".to_string(),
                range
            }
        );
    }

    #[rstest::rstest]
    #[case( // 1
        vec![
            Filter::All(vec![
                filter("x", lt(1)),
                filter("x", gt(1))
            ])
        ],
        Query::Empty
    )]
    #[case( // 2
        vec![
            Filter::All(vec![
                filter("x", lt(10)),
                filter("x", gte(1)),
            ])
        ],
        Query::Range { field: "x".to_string(), range: Range::of(1..10) },
    )]
    #[case( // 3
        vec![
            Filter::All(vec![
                filter("x", lt(10)),
                filter("x", gte(1)),
                filter("x", ScalarCondition::NotIn(vec_into![7])),
                filter("x", ScalarCondition::NotIn(vec_into![3]))
            ])
        ],
        Query::Intersection(vec![
            Query::Range { field: "x".to_string(), range: Range::of(1..10) },
            Query::Not(Box::new(Query::In { field: "x".to_string(), values: vec_into![7, 3] }))
        ])
    )]
    #[case( // 4
        vec![
            Filter::List { field: "x".to_string(), condition: ListCondition::HasAny(lt(10)) },
            Filter::List { field: "x".to_string(), condition: ListCondition::HasAny(gte(0)) },
        ],
        Query::Range { field: "x".to_string(), range: Range::of(0..10) }
    )]
    fn test_all(#[case] filters: Vec<Filter>, #[case] query: Query) {
        assert_eq!(Query::try_from(Filter::All(filters)).unwrap(), query);
    }

    #[rstest::rstest]
    #[case( // 1
        vec![
            filter("x", ScalarCondition::Eq(1.into())),
            filter("x", ScalarCondition::Eq(2.into()))
        ],
        Query::In { field: "x".to_string(), values: vec_into![1, 2] }
    )]
    #[case( // 2
        vec![
            filter("x", ScalarCondition::Eq(1.into())),
            Filter::All(vec![
                filter("x", lt(1)),
                filter("x", gt(1))
            ])
        ],
        Query::In { field: "x".to_string(), values: vec_into![1] }
    )]
    #[case( // 3
        vec![
            Filter::All(vec![
                filter("x", lt(1)),
                filter("x", gt(1))
            ])
        ],
        Query::Empty
    )]
    #[case( // 4
        vec![
            filter("x", ScalarCondition::Eq(1.into())),
            filter("x", ScalarCondition::Eq(2.into())),
            filter("y", ScalarCondition::Eq(3.into())),
            filter("y", ScalarCondition::Eq(4.into()))
        ],
        Query::Union(vec![
            Query::In { field: "x".to_string(), values: vec_into![1, 2] },
            Query::In { field: "y".to_string(), values: vec_into![3, 4] }
        ])
    )]
    #[case( // 5
        vec![
            Filter::List { field: "x".to_string(), condition: ListCondition::HasAny(ScalarCondition::Eq(1.into())) },
            Filter::List { field: "x".to_string(), condition: ListCondition::HasAny(ScalarCondition::Eq(2.into())) },
        ],
        Query::In { field: "x".to_string(), values: vec_into![1, 2] }
    )]
    fn test_any(#[case] filters: Vec<Filter>, #[case] query: Query) {
        assert_eq!(Query::try_from(Filter::Any(filters)).unwrap(), query);
    }

    #[rstest::rstest]
    #[case( // 1
        vec![
            filter("x", ScalarCondition::Eq(1.into())),
            filter("x", ScalarCondition::Eq(2.into()))
        ],
        Query::Not(Box::new(Query::In { field: "x".to_string(), values: vec_into![1, 2] }))
    )]
    #[case( // 2
        vec![
            filter("x", ScalarCondition::Eq(1.into())),
            Filter::Any(vec![
                filter("x", ScalarCondition::Eq(2.into())),
                filter("y", ScalarCondition::Eq(3.into()))
            ])
        ],
        Query::Not(Box::new(Query::Union(vec![
            Query::In { field: "x".to_string(), values: vec_into![1, 2] },
            Query::In { field: "y".to_string(), values: vec_into![3] }
        ])))
    )]
    fn test_none(#[case] filters: Vec<Filter>, #[case] query: Query) {
        assert_eq!(Query::try_from(Filter::None(filters)).unwrap(), query);
    }

    #[rstest::rstest]
    #[case( // 1
        ListCondition::IsEmpty(true),
        Query::IsNull { field: "x".to_string() }
    )]
    #[case( // 2
        ListCondition::IsEmpty(false),
        Query::Not(Box::new(Query::IsNull { field: "x".to_string() }))
    )]
    #[case( // 3
        ListCondition::HasAny(lt(10)),
        Query::Range { field: "x".to_string(), range: Range::of(..10) }
    )]
    #[case( // 4
        ListCondition::HasAny(ScalarCondition::All(vec![lt(10), gte(1)])),
        Query::Range { field: "x".to_string(), range: Range::of(1..10) }
    )]
    #[case( // 5
        ListCondition::HasNone(ScalarCondition::Any(vec![ScalarCondition::In(vec_into!(1, 2)), ScalarCondition::In(vec_into!(3, 4))])),
        Query::Not(Box::new(Query::In { field: "x".to_string(), values: vec_into![1, 2, 3, 4] }))
    )]
    fn test_list(#[case] condition: ListCondition, #[case] query: Query) {
        assert_eq!(
            Query::try_from(Filter::List {
                field: "x".to_string(),
                condition
            })
            .unwrap(),
            query
        );
    }
}
