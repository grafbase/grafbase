use super::DynaqlEnum;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum OrderByDirection {
    ASC,
    DESC,
}

impl DynaqlEnum for OrderByDirection {
    fn ty() -> &'static str {
        "OrderByDirection"
    }

    fn values() -> Vec<String> {
        vec![
            serde_json::to_string(&OrderByDirection::ASC)
                .expect("OrderByDirection is serializable")
                .trim_matches('"')
                .to_string(),
            serde_json::to_string(&OrderByDirection::DESC)
                .expect("OrderByDirection is serializable")
                .trim_matches('"')
                .to_string(),
        ]
    }
}
