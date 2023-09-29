use std::net::IpAddr;

use chrono::{NaiveTime, TimeZone, Utc};
use tantivy::query::{
    AllQuery, BooleanQuery, BoostQuery, DisjunctionMaxQuery, EmptyQuery, RangeQuery, RegexQuery, TermSetQuery,
};
use tantivy::schema::Value;
use tantivy::tokenizer::TextAnalyzer;
use tantivy::Index;
use tantivy::{
    self,
    query::{FuzzyTermQuery, Occur, PhraseQuery, Query as TantivyQuery, TermQuery},
    schema::{Field, IndexRecordOption},
    Term,
};

use combine::Parser;

use super::query::{Query, Range};
use super::utils::tokenized_field_name;
use super::{BadRequestError, SearchResult};
use runtime::search::{FieldType, ScalarValue, Schema};

pub struct TantivyQueryBuilder<'a> {
    index: &'a Index,
    schema: &'a Schema,
    typo_tolerance: TypoTolerance,
}

pub struct TypoTolerance {
    min_word_size_for_one_typo: u8,
    min_word_size_for_two_typos: u8,
}

impl Default for TypoTolerance {
    fn default() -> Self {
        // Algolia's default
        // Meilisearch uses 5 & 9
        Self {
            min_word_size_for_one_typo: 4,
            min_word_size_for_two_typos: 8,
        }
    }
}

impl TypoTolerance {
    fn supported_typos_for_word_size(&self, n: usize) -> u8 {
        if n >= self.min_word_size_for_two_typos.into() {
            2
        } else {
            u8::from(n >= self.min_word_size_for_one_typo.into())
        }
    }
}

impl<'a> TantivyQueryBuilder<'a> {
    pub(crate) fn new(index: &'a Index, schema: &'a Schema) -> Self {
        Self {
            index,
            schema,
            typo_tolerance: TypoTolerance::default(),
        }
    }

    pub(crate) fn build(&self, query: Query) -> SearchResult<Box<dyn TantivyQuery>> {
        // Inspired from Tantivy's QueryParser
        Ok(match query {
            Query::Intersection(queries) => {
                // TODO:Optimize the Not to be directly included instead of creating nested Boolean
                // queries.
                let subqueries = queries
                    .into_iter()
                    .map(|query| self.build(query).map(|q| (Occur::Must, q)))
                    .collect::<Result<Vec<_>, _>>()?;
                Box::new(BooleanQuery::new(subqueries))
            }
            Query::Union(queries) => {
                let mut terms = vec![];
                let mut subqueries = vec![];
                for query in queries {
                    match query {
                        Query::In { field, values } => {
                            let field = self.get_field(&field)?;
                            terms.extend(values.into_iter().map(|value| to_term(field, value)));
                        }
                        query => subqueries.push((Occur::Should, self.build(query)?)),
                    }
                }
                if !terms.is_empty() {
                    subqueries.push((Occur::Should, Box::new(TermSetQuery::new(terms))));
                }
                Box::new(BooleanQuery::new(subqueries))
            }
            Query::Not(query) => {
                match *query {
                    Query::IsNull { field } => {
                        if self.is_nullable_field(&field)? {
                            Box::new(self.build(Query::Range {
                                field,
                                range: Range::unbounded(),
                            })?)
                        } else {
                            Box::new(AllQuery)
                        }
                    }
                    // Imitating SQL behavior, NOT IN and NOT BETWEEN does not return NULLs.
                    query @ (Query::In { .. } | Query::Range { .. }) => {
                        let field = match &query {
                            Query::In { field, .. } | Query::Range { field, .. } => field.to_string(),
                            _ => unreachable!(),
                        };
                        Box::new(BooleanQuery::new(vec![
                            // Tantivy requires at least one query that is not MustNot
                            (Occur::Must, self.build(!Query::IsNull { field })?),
                            (Occur::MustNot, self.build(query)?),
                        ]))
                    }
                    _ => {
                        Box::new(BooleanQuery::new(vec![
                            // Tantivy requires at least one query that is not MustNot
                            (Occur::Must, Box::new(AllQuery)),
                            (Occur::MustNot, self.build(*query)?),
                        ]))
                    }
                }
            }
            Query::Range { field, range } => {
                let field = self.get_field(&field)?;
                let value_type = self.index.schema().get_field_entry(field).field_type().value_type();
                let range = range.map(|value| to_term(field, value));
                Box::new(RangeQuery::new_term_bounds(field, value_type, &range.start, &range.end))
            }
            Query::In { field, values } => {
                let field = self.get_field(&field)?;
                Box::new(TermSetQuery::new(values.into_iter().map(|value| to_term(field, value))))
            }
            Query::Regex { field, pattern } => {
                let reg = tantivy_fst::Regex::new(&pattern).map_err(|err| BadRequestError::InvalidRegex {
                    pattern,
                    err: err.to_string(),
                })?;
                Box::new(RegexQuery::from_regex(reg, self.get_field(&field)?))
            }
            Query::All => Box::new(AllQuery),
            Query::Empty => Box::new(EmptyQuery),
            Query::Text { value, fields } => self.build_text_query(&value, fields)?,
            Query::IsNull { field } => {
                if self.is_nullable_field(&field)? {
                    Box::new(BooleanQuery::new(vec![
                        // Tantivy requires at least one query that is not MustNot
                        (Occur::Must, Box::new(AllQuery)),
                        (
                            Occur::MustNot,
                            self.build(Query::Range {
                                field,
                                range: Range::unbounded(),
                            })?,
                        ),
                    ]))
                } else {
                    Box::new(EmptyQuery)
                }
            }
        })
    }

    fn build_text_query(&self, text: &str, field_names: Option<Vec<String>>) -> SearchResult<Box<dyn TantivyQuery>> {
        let field_names = field_names.unwrap_or_else(|| {
            self.schema
                .fields
                .iter()
                .filter_map(|(name, entry)| {
                    if matches!(
                        entry.ty,
                        FieldType::String { .. }
                            | FieldType::URL { .. }
                            | FieldType::Email { .. }
                            | FieldType::PhoneNumber { .. }
                    ) {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect()
        });

        // Each phrase includes one or more words
        let Ok((phrases, "")) = parser().parse(text) else {
            return Err(format!("Could not parse text: {text}").into());
        };

        let mut subqueries: Vec<Box<dyn TantivyQuery>> = vec![];
        for field_name in field_names {
            // Similar logic to Tantivy's QueryParser.
            match self.schema.fields.get(&field_name).map(|entry| &entry.ty) {
                Some(FieldType::String { .. } | FieldType::URL { .. } | FieldType::Email { .. }) => {
                    let tokenized_field = self.get_field(&tokenized_field_name(&field_name))?;
                    let tokenizer = self.get_string_tokenizer(tokenized_field)?;
                    for phrase in &phrases {
                        let mut terms_with_offset: Vec<(usize, Term)> = Vec::new();
                        tokenizer.token_stream(phrase.as_str()).process(&mut |token| {
                            let term = Term::from_field_text(tokenized_field, &token.text);
                            terms_with_offset.push((token.position, term));
                        });
                        match (terms_with_offset.len(), phrase) {
                            (0, _) => (),
                            (1, _) => {
                                for (_, term) in terms_with_offset {
                                    subqueries.push(self.build_term_query(term));
                                }
                            }
                            (_, Text::Word(word)) => {
                                // Adding full word query to boost any document with close/exact/phrase match.
                                subqueries.push(Box::new(BoostQuery::new(
                                    Box::new(DisjunctionMaxQuery::new(vec![
                                        self.build_term_query(Term::from_field_text(
                                            self.get_field(&field_name)?,
                                            word,
                                        )),
                                        Box::new(PhraseQuery::new_with_offset(terms_with_offset.clone())),
                                    ])),
                                    2.0,
                                )));
                                for (_, term) in terms_with_offset {
                                    subqueries.push(self.build_term_query(term));
                                }
                            }
                            _ => subqueries.push(Box::new(PhraseQuery::new_with_offset(terms_with_offset))),
                        }
                    }
                }
                Some(FieldType::PhoneNumber { .. }) => {
                    let term = Term::from_field_text(self.get_field(&field_name)?, text);
                    subqueries.push(self.build_term_query(term));
                }
                // Shouldn't happen unless gateway validation didn't do its job correctly
                ty => {
                    return Err(format!("Unexpected text query on field {field_name} having type {ty:?}").into());
                }
            };
        }

        Ok(Box::new(BooleanQuery::union(subqueries)))
    }

    fn build_term_query(&self, term: Term) -> Box<dyn TantivyQuery> {
        // TODO: This works well with languages using a latin alphabet as we're using the
        // AsciiFoldingFilter in our custom tokenizer. So all of those characters are mapped
        // to a single byte UTF8 byte (~ASCII). For other languages we'll just support more
        // typos than expected.
        let word_size = term.value_bytes().len();
        let typos = self.typo_tolerance.supported_typos_for_word_size(word_size);
        if typos > 0 {
            Box::new(FuzzyTermQuery::new(term, typos, true))
        } else {
            Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs))
        }
    }

    fn get_field(&self, name: &str) -> SearchResult<Field> {
        self.index
            .schema()
            .get_field(name)
            .ok_or_else(|| format!("Unknown field: '{name}'").into())
    }

    fn is_nullable_field(&self, name: &str) -> SearchResult<bool> {
        self.schema
            .fields
            .get(name)
            .ok_or_else(|| format!("Unknown field: '{name}'").into())
            .map(|field| field.ty.is_nullable())
    }

    fn get_string_tokenizer(&self, field: Field) -> SearchResult<TextAnalyzer> {
        match self.index.schema().get_field_entry(field).field_type() {
            tantivy::schema::FieldType::Str(ref str_options) => Ok(self
                .index
                .tokenizers()
                .get(
                    str_options
                        .get_indexing_options()
                        .expect("Strings are always indexed")
                        .tokenizer(),
                )
                .expect("String is always tokenized with our tokenizer")),
            _ => Err(format!(
                "Tried to retrieve the tokenzier for a non string field {}",
                self.index.schema().get_field_name(field)
            )
            .into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Text {
    Phrase(String),
    Word(String),
}

impl Text {
    fn as_str(&self) -> &str {
        match self {
            Text::Phrase(s) | Text::Word(s) => s,
        }
    }
}

// Pretty simple parser for now, we might want to allow a bit more later (-, AND, OR, etc.)
fn parser<'a>() -> impl Parser<&'a str, Output = Vec<Text>> {
    use combine::{
        many1,
        parser::char::{char, spaces},
        satisfy, sep_end_by,
    };
    let word = many1(satisfy(|c: char| !c.is_whitespace())).map(Text::Word);
    let phrase = char('"')
        .with(many1(satisfy(|c: char| c != '"')))
        .skip(char('"'))
        .map(Text::Phrase);
    spaces().with(sep_end_by(phrase.or(word), spaces()))
}

fn to_term(field: Field, value: ScalarValue) -> Term {
    use tantivy::schema::Value::{Bool, Date, IpAddr, Str, F64, I64};

    match to_tantivy(value) {
        Str(val) => Term::from_field_text(field, &val),
        I64(val) => Term::from_field_i64(field, val),
        F64(val) => Term::from_field_f64(field, val),
        Bool(val) => Term::from_field_bool(field, val),
        Date(val) => Term::from_field_date(field, val),
        IpAddr(val) => Term::from_field_ip_addr(field, val),
        _ => unreachable!("We're not using any other tantivy types for ScalarValues"),
    }
}

fn to_tantivy(value: ScalarValue) -> Value {
    use ScalarValue::{Boolean, Date, DateTime, Email, Float, IPAddress, Int, PhoneNumber, String, Timestamp, URL};
    match value {
        URL(text) | Email(text) | PhoneNumber(text) | String(text) => Value::Str(text),
        Int(val) => Value::I64(val),
        Float(val) => Value::F64(val),
        Date(date) => {
            let datetime = Utc.from_utc_datetime(&date.and_time(NaiveTime::from_hms_opt(0, 0, 0).expect("Valid time")));
            Value::Date(tantivy::DateTime::from_timestamp_millis(datetime.timestamp_millis()))
        }
        Timestamp(timestamp) => Value::Date(tantivy::DateTime::from_timestamp_millis(timestamp.timestamp_millis())),
        DateTime(datetime) => Value::Date(tantivy::DateTime::from_timestamp_millis(datetime.timestamp_millis())),
        Boolean(val) => Value::Bool(val),
        IPAddress(ip_addr) => Value::IpAddr(match ip_addr {
            IpAddr::V4(addr) => addr.to_ipv6_mapped(),
            IpAddr::V6(addr) => addr,
        }),
    }
}
