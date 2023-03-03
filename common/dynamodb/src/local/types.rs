use crate::constant::OWNED_BY;

use super::{consts::BRIDGE_PROTOCOL, utils::joined_repeating};
use chrono::{DateTime, SecondsFormat, Utc};
use dynomite::AttributeValue;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rusoto_dynamodb::Put;
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;
use std::{
    collections::{HashMap, VecDeque},
    net::Ipv4Addr,
};

pub fn serialize_dt_to_rfc3339<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339_opts(SecondsFormat::Millis, true))
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub pk: String,
    pub sk: String,
    pub gsi1sk: Option<String>,
    pub gsi1pk: Option<String>,
    pub gsi2pk: Option<String>,
    pub gsi2sk: Option<String>,
    pub entity_type: Option<String>,
    pub relation_names: Vec<String>,
    pub document: HashMap<String, AttributeValue>,
    #[serde(serialize_with = "serialize_dt_to_rfc3339")]
    pub created_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_dt_to_rfc3339")]
    pub updated_at: DateTime<Utc>,
    pub owned_by: Option<String>,
}

pub trait GetDocumentBuiltin {
    fn get_document_builtin(&self, key: DocumentBuiltin) -> String;
}

impl GetDocumentBuiltin for Put {
    fn get_document_builtin(&self, key: DocumentBuiltin) -> String {
        self.item
            .get(key.as_str())
            .and_then(|item| item.s.clone())
            .expect("must exist")
    }
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum DocumentBuiltin {
    Pk,
    Sk,
    Gsi1Pk,
    Gsi1Sk,
    Gsi2Pk,
    Gsi2Sk,
    Type,
    RelationNames,
    CreatedAt,
    UpdatedAt,
}

impl DocumentBuiltin {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Pk => "__pk",
            Self::Sk => "__sk",
            Self::Gsi1Pk => "__gsi1pk",
            Self::Gsi1Sk => "__gsi1sk",
            Self::Gsi2Pk => "__gsi2pk",
            Self::Gsi2Sk => "__gsi2sk",
            Self::Type => "__type",
            Self::RelationNames => "__relation_names",
            Self::CreatedAt => "__created_at",
            Self::UpdatedAt => "__updated_at",
        }
    }
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum Column {
    Pk,
    Sk,
    Gsi1Pk,
    Gsi1Sk,
    Gsi2Pk,
    Gsi2Sk,
    EntityType,
    RelationNames,
    Document,
    CreatedAt,
    UpdatedAt,
    OwnedBy,
}

impl Column {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pk => "pk",
            Self::Sk => "sk",
            Self::Gsi1Pk => "gsi1pk",
            Self::Gsi1Sk => "gsi1sk",
            Self::Gsi2Pk => "gsi2pk",
            Self::Gsi2Sk => "gsi2sk",
            Self::EntityType => "entity_type",
            Self::RelationNames => "relation_names",
            Self::Document => "document",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
            Self::OwnedBy => "owned_by",
        }
    }
}

pub struct Row {
    pub columns: String,
    pub values: HashMap<&'static str, SqlValue>,
    pub placeholders: String,
}

impl Row {
    pub fn from(array: [(Column, Option<String>); 12]) -> Self {
        let keyed_values = array.iter().cloned().filter_map(|(column, value)| {
            value
                .as_ref()
                .map(move |value| (column, SqlValue::String(value.clone())))
        });

        let keys: Vec<&str> = keyed_values
            .clone()
            .map(|(key, _)| key)
            .map(|key| key.as_str())
            .collect();

        let values = keyed_values.map(|(column, value)| (column.as_str(), value)).collect();

        let columns = keys.join(",");
        let placeholders = keys.iter().map(|key| format!("?{key}")).collect::<Vec<_>>().join(",");

        Self {
            columns,
            values,
            placeholders,
        }
    }

    pub fn from_record(record: Record) -> Self {
        let document = json_string(&record.document).expect("must be serializable");
        let relation_names = json_string(&record.relation_names).expect("must be serializable");
        let created_at = record.created_at.to_rfc3339_opts(SecondsFormat::Millis, true);
        let updated_at = record.updated_at.to_rfc3339_opts(SecondsFormat::Millis, true);

        let record_mapping = [
            (Column::Pk, Some(record.pk)),
            (Column::Sk, Some(record.sk)),
            (Column::Gsi1Pk, record.gsi1pk),
            (Column::Gsi1Sk, record.gsi1sk),
            (Column::Gsi2Pk, record.gsi2pk),
            (Column::Gsi2Sk, record.gsi2sk),
            (Column::EntityType, record.entity_type),
            (Column::CreatedAt, Some(created_at)),
            (Column::UpdatedAt, Some(updated_at)),
            (Column::RelationNames, Some(relation_names)),
            (Column::OwnedBy, record.owned_by),
            (Column::Document, Some(document)),
        ];

        Self::from(record_mapping)
    }
}

pub enum BridgeUrl<'a> {
    Query(&'a str),
    Mutation(&'a str),
}

impl<'a> ToString for BridgeUrl<'a> {
    fn to_string(&self) -> String {
        let (endpoint, port) = match self {
            Self::Query(port) => ("query", port),
            Self::Mutation(port) => ("mutation", port),
        };

        format!("{BRIDGE_PROTOCOL}://{}:{port}/{endpoint}", Ipv4Addr::LOCALHOST)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OperationKind {
    Constraint(Constraint),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", content = "reportingData")]
pub enum Constraint {
    Unique { values: Vec<String>, fields: Vec<String> },
}

#[derive(Serialize, Debug)]
pub struct Mutation {
    pub mutations: Vec<Operation>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub sql: String,
    pub values: Vec<String>,
    #[serde(flatten)]
    pub kind: Option<OperationKind>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolverInvocation<'a> {
    pub resolver_name: &'a str,
    pub arguments: serde_json::Value,
}

// TODO: SQL parameters are defined in different places. Those should all be defined here, creating
// a new struct "SqlQuery" which holds both the values and the generated SQL query.
// TODO: turn these tuple arguments into structs
pub enum Sql<'a> {
    /// value ordering: handled by [`Row`]
    Insert(&'a Row),
    /// values: to_add[], Row.values,
    InsertRelation(&'a Row, usize),
    /// values: document, updated_at, pk, sk, increments
    Update(Vec<&'a String>),
    /// values: pk, sk, to_remove[], to_add[], document, updated_at
    UpdateWithRelations(usize, usize),
    /// values: pk, sk, to_remove[], document, updated_at
    DeleteRelations(usize),
    /// values: pk, sk
    DeleteByIds,
    /// values: partition_keys[], sorting_keys[]
    SelectIdPairs(usize),
    /// values: pk, entity_type, edges[]
    SelectIdWithEdges(String, usize),
    /// values: pk
    SelectId(String),
    /// values: parent_pk, relation_name
    SelectSingleRelation(String),
    SelectTypePaginated {
        has_origin: bool,
        is_nested: bool,
        ascending: bool,
        edges_count: usize,
        filter_by_owner: bool,
    },
    /// values: entity_type
    SelectType,
}

#[derive(Clone, Debug)]
pub enum SqlValue {
    String(String),
    VecDeque(VecDeque<String>),
}

impl<'a> Sql<'a> {
    const TABLE: &'static str = "records";

    /// returns a minified sql query with stripped value names, and a list of ordered values accoridng to their positioning within the query.
    ///
    /// passed values are either [`SqlValue::String`] in which case they can be used an arbitrary amount of times, or
    /// [`SqlValue::VecDeque`] in which case the number of usages should match the length of the passed [`VecDeque`].
    ///
    /// keys should match the value names used in the specific query (without the leading `?`).
    ///
    /// will panic if a required key is missing. unused keys are allowed.
    ///
    /// ```no_run
    /// # use maplit::hashmmap;
    /// #
    ///  Sql::SelectTypeWithEdges(number_of_edges).compile(hashmap! {
    ///     "entity_type" => SqlValue::String(/* ... */),
    ///     "edges" => SqlValue::VecDeque(vec![/* ... */]),
    ///  })
    ///
    /// ```
    pub fn compile(&self, values: HashMap<&'a str, SqlValue>) -> (String, Vec<String>) {
        let sql_string = match self {
            Self::Insert(Row {
                columns, placeholders, ..
            }) => {
                format!(
                    "INSERT INTO {table} ({columns}) VALUES ({placeholders})",
                    table = Self::TABLE
                )
            }
            Self::InsertRelation(
                Row {
                    columns, placeholders, ..
                },
                to_add_count,
            ) => {
                let to_add = if *to_add_count > 0 {
                    format!(
                        "(SELECT * FROM original UNION VALUES {to_add_placeholders})",
                        to_add_placeholders = joined_repeating("(?to_add)", *to_add_count, ",")
                    )
                } else {
                    String::new()
                };

                format!("
                    WITH
                        original AS (SELECT json_each.value FROM {table}, json_each({table}.relation_names) WHERE pk=?pk AND sk=?sk),
                        updated AS (SELECT DISTINCT * FROM {to_add})

                    INSERT INTO 
                        {table} ({columns}) 
                    VALUES ({placeholders})
                    ON CONFLICT (pk,sk)
                    DO
                    UPDATE
                    SET
                        relation_names=(SELECT json_group_array(updated.value) FROM updated),
                        document=json_set(document, '$.__relation_names.SS', (SELECT json_group_array(updated.value) FROM updated))",
                    table = Self::TABLE,
                    to_add = to_add
                )
            }

            Self::Update(increment_fields) => {
                let document_update = if increment_fields.is_empty() {
                    "json_patch(document, ?document)".to_string()
                } else {
                    increment_fields.iter().fold(String::new(), |accumulator, current| {
                        if accumulator.is_empty() {
                            format!("json_set(json_patch(document, ?document), '$.{current}.N', cast(coalesce(json_extract(document, '$.{current}.N'), 0) + cast(?increments as NUMERIC) as TEXT))")
                        } else {
                            format!("json_set({accumulator}, '$.{current}.N', cast(coalesce(json_extract(document, '$.{current}.N'), 0) + cast(?increments as NUMERIC) as TEXT))")
                        }
                    })
                };
                format!(
                    indoc::indoc! {"
                    UPDATE {table}
                    SET 
                        document={document_update},
                        updated_at=?updated_at
                    WHERE pk=?pk AND sk=?sk
                "},
                    table = Self::TABLE,
                    document_update = document_update
                )
            }
            Self::UpdateWithRelations(to_remove_count, to_add_count) => {
                let to_remove = if *to_remove_count > 0 {
                    format!(
                        "WHERE value NOT IN ({to_remove_placeholders})",
                        to_remove_placeholders = joined_repeating("?to_remove", *to_remove_count, ",")
                    )
                } else {
                    String::new()
                };

                let to_add = if *to_add_count > 0 {
                    format!(
                        "(SELECT * FROM removed UNION VALUES {to_add_placeholders})",
                        to_add_placeholders = joined_repeating("(?to_add)", *to_add_count, ",")
                    )
                } else {
                    "removed".to_owned()
                };

                format!(
                    indoc::indoc! {"
                    WITH
                        original AS (SELECT json_each.value FROM {table}, json_each({table}.relation_names) WHERE pk=?pk AND sk=?sk),
                        removed AS (SELECT value FROM original {to_remove}),
                        updated AS (SELECT DISTINCT * FROM {to_add})
                
                    UPDATE {table} SET 
                        relation_names=(SELECT json_group_array(updated.value) FROM updated),
                        document=json_patch(document, ?document),
                        updated_at=?updated_at
                    WHERE pk=?pk AND sk=?sk
                "},
                    table = Self::TABLE,
                    to_remove = to_remove,
                    to_add = to_add
                )
            }
            Self::DeleteRelations(to_remove_count) => {
                let to_remove = if *to_remove_count > 0 {
                    format!(
                        "WHERE value NOT IN ({to_remove_placeholders})",
                        to_remove_placeholders = joined_repeating("?to_remove", *to_remove_count, ",")
                    )
                } else {
                    String::new()
                };

                format!(
                    indoc::indoc! {"
                    WITH
                        original AS (SELECT json_each.value FROM {table}, json_each({table}.relation_names) WHERE pk=?pk AND sk=?sk),
                        removed AS (SELECT value FROM original {to_remove})
                
                    UPDATE {table} SET 
                        relation_names=(SELECT json_group_array(removed.value) FROM removed),
                        document=json_patch(document, ?document),
                        updated_at=?updated_at
                    WHERE pk=?pk AND sk=?sk
                "},
                    table = Self::TABLE,
                    to_remove = to_remove,
                )
            }
            Self::DeleteByIds => {
                format!("DELETE FROM {table} WHERE pk=?pk AND sk=?sk", table = Self::TABLE)
            }
            Self::SelectIdPairs(pair_count) => {
                format!(
                    "SELECT * FROM {table} WHERE {id_pairs}",
                    table = Self::TABLE,
                    // AND has precedence over OR so no grouping needed
                    id_pairs = joined_repeating("pk=?partition_keys AND sk=?sorting_keys", *pair_count, " OR ")
                )
            }
            Self::SelectIdWithEdges(pk, number_of_edges) => {
                format!(
                    indoc::indoc! {"
                    SELECT
                        *
                    FROM
                        {table}
                    WHERE
                        {pk}=?pk
                        AND entity_type=?entity_type
                    UNION ALL
                    SELECT
                        {table}.*
                    FROM
                        {table},
                        json_each({table}.relation_names)
                    WHERE
                        {table}.pk=?pk
                        AND ({edges})
                "},
                    table = Self::TABLE,
                    pk = pk,
                    edges = joined_repeating("json_each.value=?edges", *number_of_edges, " OR "),
                )
            }
            Self::SelectId(pk) => {
                format!("SELECT * FROM {table} WHERE {pk}=?pk", table = Self::TABLE)
            }
            Self::SelectSingleRelation(pk_index) => {
                format!(
                    indoc::indoc! {"
                        SELECT 
                            * 
                        FROM 
                            {table},
                            json_each({table}.relation_names) 
                        WHERE 
                            {table}.{pk_index}=?parent_pk
                            AND json_each.value=?relation_name
                    "},
                    pk_index = pk_index,
                    table = Self::TABLE
                )
            }
            Self::SelectType => {
                format!(
                    indoc::indoc! {"
                            SELECT * from {table}
                            WHERE entity_type=?entity_type AND pk=sk
                            ORDER BY sk
                        "},
                    table = Self::TABLE,
                )
            }
            Self::SelectTypePaginated {
                has_origin,
                is_nested,
                ascending,
                edges_count,
                filter_by_owner,
            } => {
                let mut r#where = vec!["entity_type=?entity_type".to_string()];
                let select = if *is_nested {
                    r#where.push("pk=?pk".to_string());
                    r#where.push("json_each.value=?relation_name".to_string());
                    format!(
                        "SELECT {table}.* FROM {table}, json_each({table}.relation_names)",
                        table = Self::TABLE,
                    )
                } else {
                    r#where.push("pk=sk".to_string());
                    format!("SELECT * FROM {table}", table = Self::TABLE,)
                };
                if *has_origin {
                    let op = if *ascending { ">" } else { "<" };
                    r#where.push(format!("sk {op} ?sk"));
                }
                if *filter_by_owner {
                    r#where.push(format!(
                        "{owned_by_column} = ?{OWNED_BY}",
                        owned_by_column = Column::OwnedBy.as_str()
                    ));
                }
                let ordering = if *ascending { "ASC" } else { "DESC" };
                let page = format!(
                    indoc::indoc! {"
                        {select}
                        WHERE {conditions}
                        ORDER BY sk {ordering} LIMIT ?query_limit
                    "},
                    select = select,
                    conditions = r#where.join(" AND "),
                    ordering = ordering
                );
                if *edges_count == 0 {
                    page
                } else {
                    format!(
                        indoc::indoc! {r##"
                            WITH page AS (
                                {page}
                            )

                            SELECT sk AS "__#key", * FROM page
                            UNION ALL
                            SELECT {table}.pk as "__#key", {table}.*
                            FROM
                                {table},
                                json_each({table}.relation_names)
                            WHERE
                                ({table}.pk) IN (SELECT sk FROM page)
                                AND ({edges})
                            ORDER BY "__#key" {ordering}
                        "##},
                        page = page,
                        ordering = ordering,
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?edges", *edges_count, " OR "),
                    )
                }
            }
        };

        (minify(sql_string.clone()), fold_values(sql_string, values))
    }
}

static NEWLINES_AND_TABS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\n|\t)+").expect("must parse"));
static SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").expect("must parse"));
static VARIABLES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\?\w+").expect("must parse"));

fn minify(sql_string: String) -> String {
    let temp = NEWLINES_AND_TABS.replace_all(&sql_string, " ").clone();
    let temp = VARIABLES.replace_all(&temp, "?").clone();
    let minified = SPACES.replace_all(&temp, " ");

    minified.trim().to_owned()
}

// TODO: it may be possible to shift some of this work to compile time
fn fold_values(query: String, mut values: HashMap<&str, SqlValue>) -> Vec<String> {
    VARIABLES
        .find_iter(&query)
        // TODO: this map has side effects (values.pop_front()), to be refactored
        .map(|variable_match| {
            let mut name = query[variable_match.range()].to_owned();
            // remove the leading `?`
            name.remove(0);
            let values = match values.get_mut(name.as_str()).expect("must exist") {
                SqlValue::String(value) => value.clone(),
                SqlValue::VecDeque(values) => values.pop_front().expect("must exist"),
            };
            (variable_match.start(), values)
        })
        .sorted_by_key(|item| item.0)
        .map(|item| item.1)
        .collect()
}

#[test]
fn test_serde_roundtrip_record() {
    let rec = Record {
        pk: String::new(),
        sk: String::new(),
        gsi1sk: None,
        gsi1pk: None,
        gsi2pk: None,
        gsi2sk: None,
        entity_type: None,
        relation_names: Vec::new(),
        document: HashMap::new(),
        created_at: DateTime::<chrono::FixedOffset>::parse_from_rfc3339("1970-01-01T00:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc),
        updated_at: DateTime::<chrono::FixedOffset>::parse_from_rfc3339("1970-01-01T00:00:00.000Z")
            .unwrap()
            .with_timezone(&Utc),
        owned_by: None,
    };

    let serialized = serde_json::to_string(&rec);
    let roundtripped = serde_json::from_str::<Record>(&serialized.unwrap()).unwrap();
    assert_eq!(roundtripped.updated_at.timezone(), Utc);
    assert_eq!(roundtripped, rec);
}
