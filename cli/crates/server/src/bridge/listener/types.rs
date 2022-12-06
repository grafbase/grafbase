use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[allow(unused)]
#[derive(sqlx::FromRow, Debug)]
pub struct Modification {
    pub id: i64,
    pub modification_type: String,
    pub pk_old: Option<String>,
    pub sk_old: Option<String>,
    pub gsi1pk_old: Option<String>,
    pub gsi1sk_old: Option<String>,
    pub gsi2pk_old: Option<String>,
    pub gsi2sk_old: Option<String>,
    pub entity_type_old: Option<String>,
    pub relation_names_old: Option<Value>,
    pub document_old: Option<Value>,
    pub created_at_old: Option<String>,
    pub updated_at_old: Option<String>,
    pub pk_new: Option<String>,
    pub sk_new: Option<String>,
    pub gsi1pk_new: Option<String>,
    pub gsi1sk_new: Option<String>,
    pub gsi2pk_new: Option<String>,
    pub gsi2sk_new: Option<String>,
    pub entity_type_new: Option<String>,
    pub relation_names_new: Option<Value>,
    pub document_new: Option<Value>,
    pub created_at_new: Option<String>,
    pub updated_at_new: Option<String>,
}

impl Modification {
    pub fn to_event_name(&self) -> &'static str {
        match self.modification_type.as_ref() {
            "INSERT" => "INSERT",
            "UPDATE" => "MODIFY",
            "DELETE" => "REMOVE",
            _ => unreachable!(),
        }
    }

    pub fn to_keys(&self) -> Value {
        match self.modification_type.as_ref() {
            "INSERT" | "UPDATE" => {
                vec![
                    ("pk".to_owned(), self.pk_new.clone()),
                    ("sk".to_owned(), self.sk_new.clone()),
                    ("gsi1pk".to_owned(), self.gsi1pk_new.clone()),
                    ("gsi1sk".to_owned(), self.gsi1sk_new.clone()),
                    ("gsi2pk".to_owned(), self.gsi2pk_new.clone()),
                    ("gsi2sk".to_owned(), self.gsi2sk_new.clone()),
                ]
            }
            "DELETE" => {
                vec![
                    ("pk".to_owned(), self.pk_old.clone()),
                    ("sk".to_owned(), self.sk_old.clone()),
                    ("gsi1pk".to_owned(), self.gsi1pk_old.clone()),
                    ("gsi1sk".to_owned(), self.gsi1sk_old.clone()),
                    ("gsi2pk".to_owned(), self.gsi2pk_old.clone()),
                    ("gsi2sk".to_owned(), self.gsi2sk_old.clone()),
                ]
            }
            _ => unreachable!(),
        }
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key, json!({ "S": value }))))
        .collect()
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecord {
    pub aws_region: String,
    #[serde(rename = "dynamodb")]
    pub change: StreamRecord,
    #[serde(rename = "eventID")]
    pub event_id: String,
    pub event_name: String,
    pub table_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamRecord {
    #[serde(rename = "ApproximateCreationDateTime")]
    pub approximate_creation_date_time: i64,
    #[serde(default)]
    #[serde(rename = "Keys")]
    pub keys: Value,
    #[serde(default)]
    #[serde(rename = "NewImage")]
    pub new_image: Value,
    #[serde(default)]
    #[serde(rename = "OldImage")]
    pub old_image: Value,
    #[serde(rename = "SizeBytes")]
    pub size_bytes: i64,
}
