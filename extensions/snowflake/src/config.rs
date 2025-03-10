#[derive(serde::Deserialize)]
pub(crate) struct SnowflakeConfig {
    pub(crate) account: String,
    pub(crate) user: String,

    /// Meant for tests mostly.
    pub(crate) snowflake_api_url_override: Option<String>,

    pub(crate) warehouse: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) schema: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) authentication: Authentication,
}

#[derive(serde::Deserialize)]
pub(crate) enum Authentication {
    #[serde(rename = "key_pair_jwt")]
    KeyPairJwt { public_key: String, private_key: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_config_deserialization() {
        let toml_str = r#"
account = "cywwwdp-qv94952"
user = "tomhoule"

snowflake_api_url_override = "http://localhost:1000"

warehouse = "COMPUTE_WH"
database = "SNOWFLAKE_SAMPLE_DATA"
schema = "TPCH_SF1"
# role = ""

[authentication.key_pair_jwt]
public_key = "{{ env.SNOWFLAKE_PUBLIC_KEY }}"
private_key = "{{ env.SNOWFLAKE_PRIVATE_KEY }}"
"#;

        let config: SnowflakeConfig = toml::from_str(toml_str).unwrap();

        assert_eq!(config.account, "cywwwdp-qv94952");
        assert_eq!(config.user, "tomhoule");
        assert_eq!(
            config.snowflake_api_url_override,
            Some("http://localhost:1000".to_string())
        );
        assert_eq!(config.warehouse, Some("COMPUTE_WH".to_string()));
        assert_eq!(config.database, Some("SNOWFLAKE_SAMPLE_DATA".to_string()));
        assert_eq!(config.schema, Some("TPCH_SF1".to_string()));
        assert_eq!(config.role, None);

        match config.authentication {
            Authentication::KeyPairJwt {
                public_key,
                private_key,
            } => {
                assert_eq!(public_key, "{{ env.SNOWFLAKE_PUBLIC_KEY }}");
                assert_eq!(private_key, "{{ env.SNOWFLAKE_PRIVATE_KEY }}");
            }
        }
    }
}
