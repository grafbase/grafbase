use serde::de::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum OpenApiVersion {
    V2,
    V3,
    V3_1,
    Unknown(String),
}

impl<'de> serde::de::Deserialize<'de> for OpenApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Data {
            swagger: Option<String>,
            openapi: Option<String>,
        }

        let data = Data::deserialize(deserializer)?;

        match (data.swagger, data.openapi) {
            (Some(version), _) if version.starts_with('2') => Ok(OpenApiVersion::V2),
            (_, Some(version)) if version.starts_with("3.0") => Ok(OpenApiVersion::V3),
            (_, Some(version)) if version.starts_with("3.1") => Ok(OpenApiVersion::V3_1),
            (Some(version), _) | (_, Some(version)) => Ok(OpenApiVersion::Unknown(version)),
            _ => Err(D::Error::custom("Could not find any OpenAPI version fields")),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case::ancient_swagger(json!({"swagger": "1.0"}), OpenApiVersion::Unknown("1.0".into()))]
    #[case::version_2(json!({"swagger": "2.0.0"}), OpenApiVersion::V2)]
    #[case::version_3(json!({"openapi": "3.0.0"}), OpenApiVersion::V3)]
    #[case::version_3_1(json!({"openapi": "3.1.0"}), OpenApiVersion::V3_1)]
    #[case::shiny_new_version_4(json!({"openapi": "4.0.0"}), OpenApiVersion::Unknown("4.0.0".into()))]
    fn test_deserialize(#[case] input: serde_json::Value, #[case] expected: OpenApiVersion) {
        assert_eq!(serde_json::from_value::<OpenApiVersion>(input).unwrap(), expected);
    }
}
