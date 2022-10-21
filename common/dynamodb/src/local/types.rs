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
        }
    }
}

pub struct Row {
    pub columns: String,
    pub values: HashMap<&'static str, SqlValue>,
    pub placeholders: String,
}

impl Row {
    pub fn from(array: [(Column, Option<String>); 11]) -> Self {
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
        let placeholders = keys.iter().map(|key| format!("?{}", key)).collect::<Vec<_>>().join(",");

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

        format!("{}://{}:{}/{}", BRIDGE_PROTOCOL, Ipv4Addr::LOCALHOST, port, endpoint)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OperationKind {
    Constraint(Constraint),
    ByMutation(ByMutation),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", content = "reportingData")]
pub enum Constraint {
    Unique { value: String, field: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ByMutation {
    Delete,
    Update,
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

// TODO: turn these tuple arguments into structs
pub enum Sql<'a> {
    /// value ordering: handled by [`Row`]
    Insert(&'a Row),
    /// values: to_add[], Row.values,
    InsertRelation(&'a Row, usize),
    /// values: document, updated_at, pk, sk
    Update,
    /// values: document, updated_at, pk, sk, by_id
    UpdateByNonPrimary(&'a str),
    /// values: pk, sk, to_remove[], to_add[], document, updated_at
    UpdateWithRelations(usize, usize),
    /// values: pk, sk, to_remove[], document, updated_at
    DeleteRelations(usize),
    /// values: pk, sk
    DeleteByIds,
    /// values: by_id
    DeleteByNonPrimary(&'a str),
    /// values: partition_keys[], sorting_keys[]
    SelectIdPairs(usize),
    /// values: pk, entity_type, edges[]
    SelectIdWithEdges(String, usize),
    /// values: pk
    SelectId(String),
    /// values: parent_pk, relation_name
    SelectSingleRelation(String),
    /// values: entity_type, sk?, query_limit, edges[]
    SelectTypePaginatedForwardWithEdges(bool, usize, bool),
    /// values: entity_type, sk?, query_limit, edges[]
    SelectTypePaginatedBackwardWithEdges(bool, usize, bool),
    /// values: entity_type, sk?, query_limit
    SelectTypePaginatedForward(bool, bool),
    /// values: entity_type, sk?, query_limit
    SelectTypePaginatedBackward(bool, bool),
    /// values: entity_type
    SelectType,
}

#[derive(Clone)]
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

            Self::Update => {
                format!(
                    indoc::indoc! {"
                    UPDATE {table}
                    SET 
                        document=json_patch(document, ?document),
                        updated_at=?updated_at
                    WHERE pk=?pk AND sk=?sk
                "},
                    table = Self::TABLE
                )
            }
            Self::UpdateByNonPrimary(index) => {
                format!(
                    indoc::indoc! {"
                    WITH
                        primary_key AS (SELECT {index} FROM {table} WHERE pk=?by_id AND sk=?by_id LIMIT 1)

                    UPDATE {table}
                    SET 
                        document=json_patch(document, ?document),
                        updated_at=?updated_at
                    WHERE pk=(SELECT {index} FROM primary_key) AND sk=(SELECT {index} FROM primary_key)
                "},
                    table = Self::TABLE,
                    index = index
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
            Self::DeleteByNonPrimary(index) => {
                format!(
                    indoc::indoc! {"
                        WITH
                            primary_key AS (SELECT {index} FROM {table} WHERE pk=?by_id AND sk=?by_id LIMIT 1)

                        DELETE FROM {table} WHERE pk=(SELECT {index} FROM primary_key) AND sk=(SELECT {index} FROM primary_key)
                    "},
                    table = Self::TABLE,
                    index = index
                )
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
            Self::SelectTypePaginatedForwardWithEdges(has_key, number_of_edges, is_nested) => {
                let select = if *is_nested {
                    format!(
                        "SELECT {table}.* FROM {table}, json_each({table}.relation_names)",
                        table = Self::TABLE,
                    )
                } else {
                    format!("SELECT * FROM {table}", table = Self::TABLE,)
                };
                let nested_search = if *is_nested {
                    "AND json_each.value=?relation_name"
                } else {
                    ""
                };
                let pk_query = if *is_nested { "pk=?pk" } else { "pk=sk" };

                if *has_key {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} AND sk < ?sk {nested_search}
                                ORDER BY pk DESC LIMIT ?query_limit
                            )
                            
                            SELECT * FROM page
                            UNION ALL
                            SELECT {table}.* FROM 
                                {table}, 
                                json_each({table}.relation_names)
                            WHERE
                                ({table}.pk) IN (SELECT pk FROM page)
                                AND ({edges})
                            ORDER BY pk DESC
                        "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search,
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?edges", *number_of_edges, " OR "),
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} {nested_search}
                                ORDER BY pk DESC LIMIT ?query_limit
                            )
                            
                            SELECT * FROM page
                            UNION ALL
                            SELECT {table}.* FROM 
                                {table}, 
                                json_each({table}.relation_names)
                            WHERE
                                ({table}.pk) IN (SELECT pk FROM page)
                                AND ({edges})
                            ORDER BY pk DESC
                        "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search,
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?edges", *number_of_edges, " OR "),
                    )
                }
            }
            Self::SelectTypePaginatedBackwardWithEdges(has_key, number_of_edges, is_nested) => {
                let select = if *is_nested {
                    format!(
                        "SELECT {table}.* FROM {table}, json_each({table}.relation_names)",
                        table = Self::TABLE,
                    )
                } else {
                    format!("SELECT * FROM {table}", table = Self::TABLE,)
                };
                let nested_search = if *is_nested {
                    "AND json_each.value=?relation_name"
                } else {
                    ""
                };
                let pk_query = if *is_nested { "pk=?pk" } else { "pk=sk" };

                if *has_key {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} AND sk > ?sk {nested_search}
                                ORDER BY pk LIMIT ?query_limit
                            )
                            
                            SELECT * FROM page
                            UNION ALL
                            SELECT {table}.* FROM 
                                {table}, 
                                json_each({table}.relation_names)
                            WHERE
                                ({table}.pk) IN (SELECT pk FROM page)
                                AND ({edges})
                            ORDER BY pk
                        "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search,
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?edges", *number_of_edges, " OR "),
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} {nested_search}
                                ORDER BY pk LIMIT ?query_limit
                            )
                            
                            SELECT * FROM page
                            UNION ALL
                            SELECT {table}.* FROM 
                                {table}, 
                                json_each({table}.relation_names)
                            WHERE
                                ({table}.pk) IN (SELECT pk FROM page)
                                AND ({edges})
                            ORDER BY pk
                        "},
                        pk_query = pk_query,
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?edges", *number_of_edges, " OR "),
                        select = select,
                        nested_search = nested_search
                    )
                }
            }
            Self::SelectTypePaginatedForward(has_key, is_nested) => {
                let select = if *is_nested {
                    format!(
                        "SELECT {table}.* FROM {table}, json_each({table}.relation_names)",
                        table = Self::TABLE,
                    )
                } else {
                    format!("SELECT * FROM {table}", table = Self::TABLE,)
                };
                let nested_search = if *is_nested {
                    "AND json_each.value=?relation_name"
                } else {
                    ""
                };
                let pk_query = if *is_nested { "pk=?pk" } else { "pk=sk" };
                if *has_key {
                    format!(
                        indoc::indoc! {"
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} AND sk < ?sk {nested_search}
                                ORDER BY sk DESC LIMIT ?query_limit
                            "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} {nested_search}
                                ORDER BY sk DESC LIMIT ?query_limit
                            "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search
                    )
                }
            }
            Self::SelectTypePaginatedBackward(has_key, is_nested) => {
                let select = if *is_nested {
                    format!(
                        "SELECT {table}.* FROM {table}, json_each({table}.relation_names)",
                        table = Self::TABLE,
                    )
                } else {
                    format!("SELECT * FROM {table}", table = Self::TABLE,)
                };
                let nested_search = if *is_nested {
                    "AND json_each.value=?relation_name"
                } else {
                    ""
                };
                let pk_query = if *is_nested { "pk=?pk" } else { "pk=sk" };
                if *has_key {
                    format!(
                        indoc::indoc! {"
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} AND sk > ?sk {nested_search}
                                ORDER BY sk LIMIT ?query_limit
                            "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                                {select}
                                WHERE entity_type=?entity_type AND {pk_query} {nested_search}
                                ORDER BY sk LIMIT ?query_limit
                            "},
                        pk_query = pk_query,
                        select = select,
                        nested_search = nested_search
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
        pk: "".to_owned(),
        sk: "".to_owned(),
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
    };

    let serialized = serde_json::to_string(&rec);
    let roundtripped = serde_json::from_str::<Record>(&serialized.unwrap()).unwrap();
    assert_eq!(roundtripped.updated_at.timezone(), Utc);
    assert_eq!(roundtripped, rec);
}
