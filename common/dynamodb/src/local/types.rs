use super::{consts::BRIDGE_PROTOCOL, utils::joined_repeating};
use chrono::{DateTime, SecondsFormat, Utc};
use dynomite::AttributeValue;
use once_cell::sync::Lazy;
use regex::Regex;
use rusoto_dynamodb::Put;
use serde::{Deserialize, Serialize};
use serde_json::to_string as json_string;
use std::{collections::HashMap, iter, net::Ipv4Addr};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub pk: String,
    pub sk: String,
    pub gsi1sk: String,
    pub gsi1pk: String,
    pub gsi2pk: String,
    pub gsi2sk: String,
    pub entity_type: String,
    pub relation_names: Vec<String>,
    pub document: HashMap<String, AttributeValue>,
    pub created_at: DateTime<Utc>,
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
    pub const fn as_str(&self) -> &str {
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
    pub values: Vec<String>,
    pub placeholders: String,
}

impl Row {
    pub fn from(array: &[(Column, String)]) -> Self {
        let (keys, values): (Vec<Column>, Vec<String>) = array.iter().cloned().unzip();

        let columns = keys.iter().map(Column::as_str).collect::<Vec<_>>().join(",");
        let placeholders = iter::repeat("?").take(keys.len()).collect::<Vec<_>>().join(",");

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

        Self::from(&[
            (Column::Pk, record.pk),
            (Column::Sk, record.sk),
            (Column::Gsi1Pk, record.gsi1pk),
            (Column::Gsi1Sk, record.gsi1sk),
            (Column::Gsi2Pk, record.gsi2pk),
            (Column::Gsi2Sk, record.gsi2sk),
            (Column::EntityType, record.entity_type),
            (Column::CreatedAt, created_at),
            (Column::UpdatedAt, updated_at),
            (Column::RelationNames, relation_names),
            (Column::Document, document),
        ])
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

#[derive(Serialize)]
pub struct BridgePayload<'a> {
    pub query: &'a str,
    pub variables: &'a Vec<String>,
}

pub enum Sql<'a> {
    /// value ordering: handled by [`Row`]
    Insert(&'a Row),
    /// value ordering: pk, sk, relation_names, Row values
    InsertRelation(&'a Row, usize),
    /// value ordering: document, updated_at, pk, sk
    Update,
    /// value ordering: pk, sk, vec of values to remove, vec of values to add, document, updated_at, pk, sk
    UpdateWithRelations(usize, usize),
    /// value ordering: pk, sk, vec of values to remove, document, updated_at, pk, sk
    DeleteRelations(usize),
    /// value ordering: pk, sk
    DeleteByIds,
    /// value ordering: vec of alternating pk, sk
    SelectIdPairs(usize),
    /// value ordering: pk, entity_type, pk, edges
    SelectIdWithEdges(String, usize),
    /// value ordering: pk
    SelectId(String),
    /// value ordering: entity_type, sk if has key, limit
    SelectTypePaginatedForward(bool),
    /// value ordering: entity_type, sk if has key, limit
    SelectTypePaginatedBackward(bool),
    /// value ordering: entity_type, sk if has key, limit, edges
    SelectTypePaginatedForwardWithEdges(bool, usize),
    /// value ordering: entity_type, sk if has key, limit, edges
    SelectTypePaginatedBackwardWithEdges(bool, usize),
    /// value ordering: entity_type
    SelectType,
    /// value ordering: entity_type, vec of edges
    SelectTypeWithEdges(usize),
}

impl<'a> Sql<'a> {
    const TABLE: &'static str = "records";
}

impl<'a> ToString for Sql<'a> {
    fn to_string(&self) -> String {
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
                format!("
                    WITH
                        original AS (SELECT json_each.value FROM {table}, json_each({table}.relation_names) WHERE pk=? AND sk=?),
                        updated AS (SELECT DISTINCT * FROM (SELECT * FROM original UNION VALUES {to_add}))

                    INSERT INTO 
                        {table} ({columns}) 
                    VALUES ({placeholders})
                    ON CONFLICT (pk,sk)
                    DO
                    UPDATE
                    SET
                        relation_names=(SELECT json_group_array(updated.value) FROM updated),
                        document=json_insert(document, '$.__relation_names.SS[#]', (SELECT json_group_array(updated.value) FROM updated))",
                    table = Self::TABLE,
                    to_add = joined_repeating("(?)", *to_add_count, ",")
                )
            }

            Self::Update => format!(
                indoc::indoc! {"
                    UPDATE {table}
                    SET 
                        document=json_patch(document, ?),
                        updated_at=?
                    WHERE pk=? AND sk=?
                "},
                table = Self::TABLE
            ),
            Self::UpdateWithRelations(to_remove_count, to_add_count) => format!(
                indoc::indoc! {"
                    WITH
                        original AS (SELECT json_each.value FROM {table}, json_each({table}.relation_names) WHERE pk=? AND sk=?),
                        removed AS (SELECT value FROM original WHERE value NOT IN ({to_remove})),
                        updated AS (SELECT DISTINCT * FROM (SELECT * FROM removed UNION VALUES {to_add}))
                
                    UPDATE {table} SET 
                        relation_names=(SELECT json_group_array(updated.value) FROM updated),
                        document=json_patch(document, ?),
                        updated_at=?
                    WHERE pk=? AND sk=?
                "},
                table = Self::TABLE,
                to_remove = joined_repeating("?", *to_remove_count, ","),
                to_add = joined_repeating("(?)", *to_add_count, ",")
            ),
            Self::DeleteRelations(to_remove_count) => format!(
                indoc::indoc! {"
                    WITH
                        original AS (SELECT json_each.value FROM {table}, json_each({table}.relation_names) WHERE pk=? AND sk=?),
                        removed AS (SELECT value FROM original WHERE value NOT IN ({to_remove})),
                
                    UPDATE {table} SET 
                        relation_names=(SELECT json_group_array(removed.value) FROM removed),
                        document=json_patch(document, ?),
                        updated_at=?
                    WHERE pk=? AND sk=?
                "},
                table = Self::TABLE,
                to_remove = joined_repeating("?", *to_remove_count, ","),
            ),
            Self::DeleteByIds => format!("DELETE FROM {table} WHERE pk=? AND sk=?", table = Self::TABLE),
            Self::SelectIdPairs(pair_count) => format!(
                "SELECT * FROM {table} WHERE {id_pairs}",
                table = Self::TABLE,
                // AND has precedence over OR so no grouping needed
                id_pairs = joined_repeating("pk=? AND sk=?", *pair_count, " OR ")
            ),
            Self::SelectIdWithEdges(pk, number_of_edges) => format!(
                indoc::indoc! {"
                    SELECT
                        *
                    FROM
                        {table}
                    WHERE
                        {pk}=?
                        AND entity_type=?
                    UNION ALL
                    SELECT
                        {table}.*
                    FROM
                        {table},
                        json_each({table}.relation_names)
                    WHERE
                        {table}.pk=?
                        AND ({edges})
                "},
                table = Self::TABLE,
                pk = pk,
                edges = joined_repeating("json_each.value=?", *number_of_edges, " OR "),
            ),
            Self::SelectId(pk) => format!("SELECT * FROM {table} WHERE {pk}=?", table = Self::TABLE),
            Self::SelectType => {
                format!(
                    indoc::indoc! {"
                            SELECT * from {table}
                            WHERE entity_type=? AND pk=sk
                            ORDER BY pk
                        "},
                    table = Self::TABLE,
                )
            }
            Self::SelectTypeWithEdges(number_of_edges) => {
                format!(
                    indoc::indoc! {"
                            WITH entities AS (
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk
                                ORDER BY pk
                            )
                            
                            SELECT * FROM entities
                            UNION ALL
                            SELECT {table}.* FROM 
                                {table}, 
                                json_each({table}.relation_names)
                            WHERE
                                ({table}.pk) IN (SELECT pk FROM entities)
                                AND ({edges})
                            ORDER BY pk DESC
                        "},
                    table = Self::TABLE,
                    edges = joined_repeating("json_each.value=?", *number_of_edges, " OR "),
                )
            }
            Self::SelectTypePaginatedForwardWithEdges(has_key, number_of_edges) => {
                if *has_key {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk AND sk < ?
                                ORDER BY pk DESC LIMIT ?
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
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?", *number_of_edges, " OR "),
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk
                                ORDER BY pk DESC LIMIT ?
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
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?", *number_of_edges, " OR "),
                    )
                }
            }
            Self::SelectTypePaginatedBackwardWithEdges(has_key, number_of_edges) => {
                if *has_key {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk AND sk > ?
                                ORDER BY pk LIMIT ?
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
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?", *number_of_edges, " OR "),
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                            WITH page AS (
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk
                                ORDER BY pk LIMIT ?
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
                        table = Self::TABLE,
                        edges = joined_repeating("json_each.value=?", *number_of_edges, " OR "),
                    )
                }
            }
            Self::SelectTypePaginatedForward(has_key) => {
                if *has_key {
                    format!(
                        indoc::indoc! {"
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk AND sk < ?
                                ORDER BY pk DESC LIMIT ?
                            "},
                        table = Self::TABLE,
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk
                                ORDER BY pk DESC LIMIT ?
                            "},
                        table = Self::TABLE,
                    )
                }
            }
            Self::SelectTypePaginatedBackward(has_key) => {
                if *has_key {
                    format!(
                        indoc::indoc! {"
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk AND sk > ?
                                ORDER BY pk LIMIT ?
                            "},
                        table = Self::TABLE
                    )
                } else {
                    format!(
                        indoc::indoc! {"
                                SELECT * FROM {table}
                                WHERE entity_type=? AND pk=sk
                                ORDER BY pk LIMIT ?
                            "},
                        table = Self::TABLE
                    )
                }
            }
        };

        minify(sql_string)
    }
}

static NEWLINES_AND_TABS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\n|\t)+").expect("must parse"));
static SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").expect("must parse"));

fn minify(sql_string: String) -> String {
    let temp = NEWLINES_AND_TABS.replace_all(&sql_string, " ").clone();
    let minified = SPACES.replace_all(&temp, " ");

    minified.trim().to_owned()
}

impl<'a> From<Sql<'a>> for String {
    fn from(sql: Sql<'a>) -> Self {
        sql.to_string()
    }
}
