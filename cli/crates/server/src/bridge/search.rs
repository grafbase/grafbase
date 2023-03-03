use std::net::{IpAddr, Ipv6Addr};

use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use futures_util::TryStreamExt;
use serde_json::Value;
use sqlx::SqlitePool;

use tantivy::collector::TopDocs;
use tantivy::directory::RamDirectory;
use tantivy::query::{QueryParser, QueryParserError};
use tantivy::schema::{Field, Schema, INDEXED, STORED, STRING, TEXT};
use tantivy::store::Compressor;
use tantivy::{DocAddress, Document, IndexSettings, Score, TantivyError};

use crate::bridge::types::SearchScalar;

use super::errors::ApiError;
use super::types::{RecordDocument, SearchField, SearchSchema};

const DATE_FORMAT: &str = "%Y-%m-%d";

impl From<TantivyError> for ApiError {
    fn from(error: TantivyError) -> Self {
        error!("TantivyError: {error}");
        Self::ServerError
    }
}

impl From<QueryParserError> for ApiError {
    fn from(error: QueryParserError) -> Self {
        error!("Tantivy QueryParserError: {error}");
        Self::ServerError
    }
}

pub struct Index {
    inner: tantivy::Index,
    default_fields: Vec<Field>,
    id_field: Field,
}

impl Index {
    pub fn search_top_records(&self, query: &str, limit: usize) -> Result<Vec<String>, ApiError> {
        let searcher = self.inner.reader()?.searcher();
        let query_parser = QueryParser::for_index(&self.inner, self.default_fields.clone());
        let tantivy_query = query_parser.parse_query(query)?;

        let top_docs: Vec<(Score, DocAddress)> = searcher.search(&tantivy_query, &TopDocs::with_limit(limit))?;

        let mut matching_records = vec![];
        for (_score, doc_address) in top_docs {
            // Retrieve the actual content of documents given its `doc_address`.
            let retrieved_doc = searcher.doc(doc_address)?;
            matching_records.push(
                String::from_utf8(
                    retrieved_doc
                        .get_first(self.id_field)
                        .and_then(tantivy::schema::Value::as_bytes)
                        .expect("Id is always present")
                        .to_vec(),
                )
                .expect("Id is a valid string"),
            );
        }

        trace!(
            "Found {} top documents:\n{:?}",
            matching_records.len(),
            matching_records
        );
        Ok(matching_records)
    }

    pub async fn build(pool: &SqlitePool, entity_type: &str, search_schema: &SearchSchema) -> Result<Index, ApiError> {
        trace!("Building index for {entity_type} and schema:\n{search_schema:?}");

        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_bytes_field("id", STORED);
        let (fields, default_fields) = {
            let mut all = vec![];
            let mut defaults = vec![];

            for search_field in &search_schema.fields {
                use SearchScalar::{
                    Boolean, Date, DateTime, Email, Float, IPAddress, Int, PhoneNumber, String, Timestamp, URL,
                };
                let field = match search_field.scalar {
                    URL | Email | PhoneNumber => schema_builder.add_text_field(&search_field.name, STRING),
                    String => schema_builder.add_text_field(&search_field.name, TEXT),
                    Date | DateTime | Timestamp => schema_builder.add_date_field(&search_field.name, INDEXED),
                    Int => schema_builder.add_i64_field(&search_field.name, INDEXED),
                    Float => schema_builder.add_f64_field(&search_field.name, INDEXED),
                    Boolean => schema_builder.add_bool_field(&search_field.name, INDEXED),
                    IPAddress => schema_builder.add_ip_addr_field(&search_field.name, INDEXED),
                };
                match search_field.scalar {
                    String | URL | Email | PhoneNumber => defaults.push(field),
                    _ => (),
                };
                all.push((search_field, field));
            }
            (all, defaults)
        };
        let schema = schema_builder.build();
        let builder = tantivy::Index::builder()
            .schema(schema.clone())
            .settings(IndexSettings {
                docstore_compression: Compressor::None,
                ..Default::default()
            });

        let index = builder.open_or_create(RamDirectory::create())?;
        let mut index_writer = index.writer_with_num_threads(1, 20_000_000)?;

        let mut fut = sqlx::query_as(
            r#"
        SELECT pk AS id, document
        FROM records WHERE entity_type = $1 AND pk = sk
        "#,
        )
        .bind(entity_type)
        .fetch(pool);

        let mut record_count: usize = 0;
        while let Some::<RecordDocument>(record) = fut.try_next().await? {
            record_count += 1;
            let mut doc = Document::default();
            doc.add_bytes(id_field, record.id.as_bytes());
            for (search_field, field) in &fields {
                add_field(&mut doc, *field, search_field, &record).map_err(|err| {
                    error!(
                        "{:?} for record '{}' on field '{}'",
                        err, &record.id, &search_field.name
                    );
                    ApiError::ServerError
                })?;
            }
            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
        trace!("Indexed {record_count} documents.");

        Ok(Index {
            inner: index,
            default_fields,
            id_field,
        })
    }
}

fn add_field(
    doc: &mut Document,
    field: Field,
    SearchField { name, scalar }: &SearchField,
    RecordDocument { document, .. }: &RecordDocument,
) -> Result<(), FieldError> {
    if let Some(value) = document.get(name) {
        use SearchScalar::{
            Boolean, Date, DateTime, Email, Float, IPAddress, Int, PhoneNumber, String, Timestamp, URL,
        };
        match scalar {
            URL | Email | PhoneNumber | String => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_text(field, DynamoItemExt::to_str(value)?);
                }
            }
            Int => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_i64(field, DynamoItemExt::to_i64(value)?);
                }
            }
            Float => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_f64(field, DynamoItemExt::to_f64(value)?);
                }
            }
            DateTime => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_date(
                        field,
                        tantivy::DateTime::from_timestamp_millis(DynamoItemExt::to_datetime(value)?.timestamp_millis()),
                    );
                }
            }
            Date => {
                for value in DynamoItemExt::flatten(value) {
                    let value = Utc.from_utc_datetime(
                        &DynamoItemExt::to_date(value)?.and_time(NaiveTime::from_hms_opt(0, 0, 0).expect("Valid time")),
                    );
                    doc.add_date(
                        field,
                        tantivy::DateTime::from_timestamp_millis(value.timestamp_millis()),
                    );
                }
            }
            Timestamp => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_date(
                        field,
                        tantivy::DateTime::from_timestamp_millis(
                            DynamoItemExt::to_timestamp(value)?.timestamp_millis(),
                        ),
                    );
                }
            }
            Boolean => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_bool(field, DynamoItemExt::to_bool(value)?);
                }
            }
            IPAddress => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_ip_addr(field, DynamoItemExt::to_ipaddr(value)?);
                }
            }
        };
    }
    Ok(())
}

#[derive(Debug)]
pub enum FieldError {
    MissingValue,
    InvalidValue,
}

type FieldResult<T> = Result<T, FieldError>;

struct DynamoItemExt;
impl DynamoItemExt {
    fn flatten(value: &Value) -> Vec<&Value> {
        value
            .get("L")
            .and_then(serde_json::Value::as_array)
            .map(|array| array.iter().flat_map(DynamoItemExt::flatten).collect::<Vec<_>>())
            .unwrap_or(vec![value])
    }
    fn to_str(value: &Value) -> FieldResult<&str> {
        value
            .get("S")
            .ok_or(FieldError::MissingValue)?
            .as_str()
            .ok_or(FieldError::InvalidValue)
    }

    fn to_i64(value: &Value) -> FieldResult<i64> {
        value
            .get("N")
            .ok_or(FieldError::MissingValue)?
            .as_str()
            .ok_or(FieldError::InvalidValue)?
            .parse::<i64>()
            .map_err(|_| FieldError::InvalidValue)
    }

    fn to_u64(value: &Value) -> FieldResult<u64> {
        value
            .get("N")
            .ok_or(FieldError::MissingValue)?
            .as_str()
            .ok_or(FieldError::InvalidValue)?
            .parse::<u64>()
            .map_err(|_| FieldError::InvalidValue)
    }

    fn to_f64(value: &Value) -> FieldResult<f64> {
        value
            .get("N")
            .ok_or(FieldError::MissingValue)?
            .as_str()
            .ok_or(FieldError::InvalidValue)?
            .parse::<f64>()
            .map_err(|_| FieldError::InvalidValue)
    }

    fn to_bool(value: &Value) -> FieldResult<bool> {
        value
            .get("BOOL")
            .ok_or(FieldError::MissingValue)?
            .as_bool()
            .ok_or(FieldError::InvalidValue)
    }

    fn to_datetime(value: &Value) -> FieldResult<DateTime<Utc>> {
        DynamoItemExt::to_str(value)?
            .parse::<DateTime<Utc>>()
            .map_err(|_| FieldError::InvalidValue)
    }

    fn to_date(value: &Value) -> FieldResult<NaiveDate> {
        NaiveDate::parse_from_str(DynamoItemExt::to_str(value)?, DATE_FORMAT).map_err(|_| FieldError::InvalidValue)
    }

    fn to_timestamp(value: &Value) -> FieldResult<DateTime<Utc>> {
        i64::try_from(DynamoItemExt::to_u64(value)? * 1_000_000)
            .map_err(|_| FieldError::InvalidValue)
            .map(|nanos| Utc.timestamp_nanos(nanos))
    }

    fn to_ipaddr(value: &Value) -> FieldResult<Ipv6Addr> {
        DynamoItemExt::to_str(value)?
            .parse::<IpAddr>()
            .map_err(|_| FieldError::InvalidValue)
            .map(|ip| match ip {
                IpAddr::V4(addr) => addr.to_ipv6_mapped(),
                IpAddr::V6(addr) => addr,
            })
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_dynamo_item_ext() {
        let record: Value = serde_json::from_str(
            r#"
            {
              "date": {"S": "2007-12-03"},
              "float": {"N": "98.293"},
              "text": {"S": "Dogs are the best!"},
              "int": {"N": "8179"},
              "timestamp": {"N": "1451653820000"},
              "ip": {"S": "127.0.0.1"},
              "datetime": {"S": "2016-01-01T13:10:20+00:00"},
              "bool": {"BOOL": true}
            }
            "#,
        )
        .unwrap();
        assert_eq!(
            DynamoItemExt::to_str(record.get("text").unwrap()).unwrap(),
            "Dogs are the best!"
        );
        assert_eq!(DynamoItemExt::to_i64(record.get("int").unwrap()).unwrap(), 8179);
        assert!((DynamoItemExt::to_f64(record.get("float").unwrap()).unwrap() - 98.293).abs() < f64::EPSILON);
        assert!(DynamoItemExt::to_bool(record.get("bool").unwrap()).unwrap());
        assert_eq!(
            DynamoItemExt::to_timestamp(record.get("timestamp").unwrap()).unwrap(),
            Utc.timestamp_nanos(1_451_653_820_000_000_000i64)
        );
        assert_eq!(
            DynamoItemExt::to_date(record.get("date").unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2007, 12, 3).unwrap()
        );
        assert_eq!(
            DynamoItemExt::to_datetime(record.get("datetime").unwrap()).unwrap(),
            "2016-01-01T13:10:20.000Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(
            DynamoItemExt::to_ipaddr(record.get("ip").unwrap()).unwrap().to_string(),
            Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped().to_string()
        );

        let value: Value = serde_json::from_str(
            r#"
            {"L": [{"L": [{"S": "first"}]}, {"L": [{"S": "second"}]}]}
        "#,
        )
        .unwrap();
        let result = DynamoItemExt::flatten(&value)
            .iter()
            .map(|value| DynamoItemExt::to_str(value).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(result, vec!["first", "second"]);
    }
}
