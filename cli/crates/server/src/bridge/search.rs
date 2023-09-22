use axum::extract::State;
use axum::Json;
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use common::environment::Project;
use futures_util::TryStreamExt;
use serde_json::Value;
use sqlx::SqlitePool;

use tantivy::schema::Field;
use tantivy::Document;
use ulid::Ulid;

use std::net::{IpAddr, Ipv6Addr};
use std::sync::Arc;

use super::api_counterfeit::registry::{Registry, VersionedRegistry};
use super::api_counterfeit::search::{
    self, PaginatedHits, Pagination, Query, QueryError, QueryRequest, QueryResponse, TantivyQueryBuilder,
    TopDocsPaginatedSearcher,
};
use super::errors::ApiError;
use super::server::HandlerState;
use super::types::RecordDocument;

const DATE_FORMAT: &str = "%Y-%m-%d";
const DOCUMENT_FIELD_CREATED_AT: &str = "__created_at";
const DOCUMENT_FIELD_UPDATED_AT: &str = "__updated_at";

pub struct Index<'a> {
    inner: tantivy::Index,
    schema: &'a search::Schema,
    id_field: Field,
}

impl<'a> Index<'a> {
    // needless_pass_by_value: Complains about pagination argument which has no other purpose anyway
    // cast_possible_truncation: Complains about u64 -> usize, which shouldn't matter for anything sensible.
    #[allow(clippy::needless_pass_by_value, clippy::cast_possible_truncation)]
    pub fn search(&self, query: Query, pagination: Pagination) -> Result<PaginatedHits<Ulid>, QueryError> {
        trace!("Executing query: {query:?}");
        let query = TantivyQueryBuilder::new(&self.inner, self.schema).build(query)?;
        let searcher = TopDocsPaginatedSearcher {
            searcher: self.inner.reader()?.searcher(),
            query,
            id_field: self.id_field,
            pagination_limit: 1000,
        };
        let hits: PaginatedHits<Vec<u8>> = match pagination {
            Pagination::Forward { first, after: None } => searcher.search_forward(first as usize)?,
            Pagination::Forward {
                first,
                after: Some(after),
            } => searcher.search_forward_after(first as usize, &after.try_into()?)?,
            Pagination::Backward { last, before } => {
                searcher.search_backward_before(last as usize, &before.try_into()?)?
            }
        };
        Ok(hits.map_id(|id| {
            let mut id = String::from_utf8(id).unwrap();
            // removing the prefix 'post#<ulid>' from the id
            let _ = id.drain(..(id.len() - 26));
            Ulid::from_string(&id).unwrap()
        }))
    }

    pub async fn build(
        pool: &SqlitePool,
        entity_type: &str,
        config: &'a search::Config,
    ) -> Result<Index<'a>, QueryError> {
        let schema = &config
            .indices
            .get(entity_type)
            .ok_or_else(|| {
                error!("Unknown index: {entity_type}");
                QueryError::ServerError
            })?
            .schema;

        trace!("Building index for {entity_type} and schema:\n{schema:?}");
        let (index, fields) = search::open_index(schema)?;
        let id_field = index.schema().get_field(search::ID_FIELD).unwrap();

        let mut writer = index.writer_with_num_threads(1, 20_000_000)?;
        // FIXME: GB-3636 Implement DynamoDB variant
        let mut fut = sqlx::query_as(
            r#"
        SELECT pk AS id, document
        FROM records WHERE entity_type = $1 AND pk = sk
        "#,
        )
        .bind(entity_type)
        .fetch(pool);

        let mut record_count: usize = 0;
        while let Some::<RecordDocument>(record) = fut
            .try_next()
            .await
            .map_err(|err| format!("Failed loading documents: {err:?}"))?
        {
            record_count += 1;
            let mut doc = Document::default();
            doc.add_bytes(id_field, record.id.as_bytes());
            for field in &fields {
                add_field(&mut doc, field, &record).map_err(|err| {
                    error!("{:?} for record '{}' on field '{}'", err, &record.id, &field.name);
                    QueryError::ServerError
                })?;
            }
            writer.add_document(doc)?;
        }
        writer.commit()?;
        trace!("Indexed {record_count} documents.");

        Ok(Index {
            inner: index,
            schema,
            id_field,
        })
    }
}

fn add_field(
    doc: &mut Document,
    search::IndexedField {
        name,
        doc_key,
        tokenized_doc_key,
        ty,
    }: &search::IndexedField,
    RecordDocument { document, .. }: &RecordDocument,
) -> Result<(), FieldError> {
    let document_field_name = match name.as_str() {
        "createdAt" => DOCUMENT_FIELD_CREATED_AT,
        "updatedAt" => DOCUMENT_FIELD_UPDATED_AT,
        name => name,
    };
    if let Some(value) = document.get(document_field_name) {
        use search::FieldType::{
            Boolean, Date, DateTime, Email, Float, IPAddress, Int, PhoneNumber, String, Timestamp, URL,
        };
        let field = *doc_key;
        match ty {
            URL { .. } | Email { .. } | String { .. } => {
                let tokenized_doc_key = tokenized_doc_key.unwrap();
                for value in DynamoItemExt::flatten(value) {
                    let value = DynamoItemExt::to_str(value)?;
                    doc.add_text(field, value);
                    doc.add_text(tokenized_doc_key, value);
                }
            }
            PhoneNumber { .. } => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_text(field, DynamoItemExt::to_str(value)?);
                }
            }
            Int { .. } => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_i64(field, DynamoItemExt::to_i64(value)?);
                }
            }
            Float { .. } => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_f64(field, DynamoItemExt::to_f64(value)?);
                }
            }
            DateTime { .. } => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_date(
                        field,
                        tantivy::DateTime::from_timestamp_millis(DynamoItemExt::to_datetime(value)?.timestamp_millis()),
                    );
                }
            }
            Date { .. } => {
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
            Timestamp { .. } => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_date(
                        field,
                        tantivy::DateTime::from_timestamp_millis(
                            DynamoItemExt::to_timestamp(value)?.timestamp_millis(),
                        ),
                    );
                }
            }
            Boolean { .. } => {
                for value in DynamoItemExt::flatten(value) {
                    doc.add_bool(field, DynamoItemExt::to_bool(value)?);
                }
            }
            IPAddress { .. } => {
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
    fn is_null(value: &Value) -> bool {
        value
            .get("NULL")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default()
    }

    fn flatten(value: &Value) -> Vec<&Value> {
        if DynamoItemExt::is_null(value) {
            vec![]
        } else {
            value
                .get("L")
                .and_then(serde_json::Value::as_array)
                .map(|array| array.iter().flat_map(DynamoItemExt::flatten).collect::<Vec<_>>())
                .unwrap_or(vec![value])
        }
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

pub async fn search_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, ApiError> {
    let project = Project::get();

    let registry: Registry = {
        let versioned = tokio::task::spawn_blocking::<_, Result<VersionedRegistry, ApiError>>(|| {
            let registry_value = project.registry().map_err(|err| {
                error!("Failed to read registry: {err:?}");
                ApiError::ServerError
            })?;

            serde_json::from_value::<VersionedRegistry>(registry_value).map_err(|err| {
                error!("Failed to deserialize registry: {err:?}");
                ApiError::ServerError
            })
        })
        .await
        .map_err(|e| {
            error!("Failed do read json registry: {e:?}");
            ApiError::ServerError
        })??;

        versioned.registry
    };

    let result = Index::build(&handler_state.pool, &request.index, &registry.search_config)
        .await
        .and_then(|index| index.search(request.query, request.pagination));
    Ok(Json(QueryResponse::V1(result)))
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
