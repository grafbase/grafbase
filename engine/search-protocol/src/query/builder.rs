use std::collections::BTreeMap;

use super::{BadRequestError, Query, Range, ScalarValue};

#[derive(Debug, Default)]
pub struct UnionQueryBuilder {
    // BTreeMap makes the traversal order deterministic, making equality for tests simpler.
    field_range: BTreeMap<String, Range<ScalarValue>>,
    field_in: BTreeMap<String, Vec<ScalarValue>>,
    other: Vec<Query>,
}

impl UnionQueryBuilder {
    pub fn build_from<Iter, Item>(iter: Iter) -> Result<Query, BadRequestError>
    where
        Item: TryInto<Query, Error = BadRequestError>,
        Iter: IntoIterator<Item = Item>,
    {
        let mut builder = Self::default();
        for item in iter {
            builder.add(item.try_into()?)?;
        }
        Ok(builder.build())
    }

    fn build(self) -> Query {
        let UnionQueryBuilder {
            field_range,
            field_in,
            other,
        } = self;
        if other.iter().any(|query| matches!(query, Query::All)) {
            return Query::All;
        }
        let mut queries = other
            .into_iter()
            .filter(|query| !matches!(query, Query::Empty))
            .collect::<Vec<_>>();

        for (field, range) in field_range {
            queries.push(Query::Range { field, range });
        }

        for (field, values) in field_in {
            queries.push(Query::In { field, values });
        }

        match queries.len() {
            0 => Query::Empty, // We only removed sub-queries matching Empty.
            1 => queries.into_iter().next().expect("size == 1"),
            _ => Query::Union(queries),
        }
    }

    fn add(&mut self, query: Query) -> Result<(), BadRequestError> {
        match query {
            Query::Union(queries) => {
                for query in queries {
                    self.add(query)?;
                }
            }
            Query::In { field, values } => {
                self.field_in.entry(field).or_default().extend(values);
            }
            query => self.other.push(query),
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct IntersectionQueryBuilder {
    // BTreeMap makes the traversal order deterministic, making equality for tests simpler.
    field_range: BTreeMap<String, Range<ScalarValue>>,
    field_not_in: BTreeMap<String, Vec<ScalarValue>>,
    other: Vec<Query>,
}

impl IntersectionQueryBuilder {
    pub fn build_from<Iter, Item>(iter: Iter) -> Result<Query, BadRequestError>
    where
        Item: TryInto<Query, Error = BadRequestError>,
        Iter: IntoIterator<Item = Item>,
    {
        let mut builder = Self::default();
        for item in iter {
            builder.add(item.try_into()?)?;
        }
        Ok(builder.build())
    }

    fn build(self) -> Query {
        let IntersectionQueryBuilder {
            field_range,
            field_not_in,
            other,
        } = self;
        if other.iter().any(|query| matches!(query, Query::Empty)) {
            return Query::Empty;
        }
        let mut queries = other
            .into_iter()
            .filter(|query| !matches!(query, Query::All))
            .collect::<Vec<_>>();

        for (field, range) in field_range {
            if range.is_empty() {
                return Query::Empty;
            }
            queries.push(Query::Range { field, range });
        }

        for (field, values) in field_not_in {
            queries.push(!Query::In { field, values });
        }

        match queries.len() {
            0 => Query::All, // We only removed sub-queries matching All.
            1 => queries.into_iter().next().expect("size == 1"),
            _ => Query::Intersection(queries),
        }
    }

    fn add(&mut self, query: Query) -> Result<(), BadRequestError> {
        match query {
            Query::Intersection(queries) => {
                for query in queries {
                    self.add(query)?;
                }
            }
            Query::Not(query) => match *query {
                Query::In { field, values } => {
                    self.field_not_in.entry(field).or_default().extend(values);
                }
                q => self.other.push(!q),
            },
            Query::Range { field, range } => {
                let current = self.field_range.remove(&field).unwrap_or_default();
                self.field_range.insert(
                    field.clone(),
                    Range::intersection(&current, &range).ok_or_else(|| BadRequestError::IncompatibleRanges {
                        a: Box::new(current.clone()),
                        b: Box::new(range.clone()),
                    })?,
                );
            }
            query => self.other.push(query),
        }
        Ok(())
    }
}
