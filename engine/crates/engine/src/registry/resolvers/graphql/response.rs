use http::StatusCode;
use serde_json::Value;

use super::Error;
use crate::ServerError;

#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct UpstreamResponse {
    pub data: serde_json::Value,
    pub errors: Vec<ServerError>,
}

impl UpstreamResponse {
    pub fn from_response_text(
        http_status: StatusCode,
        response_text_result: Result<String, impl Into<Error>>,
    ) -> Result<Self, Error> {
        let response_text =
            response_text_result.map_err(|error| handle_error_after_response(http_status, error, None))?;

        serde_json::from_str::<UpstreamResponse>(&response_text).map_err(|error| {
            handle_error_after_response(
                http_status,
                Error::JsonDecodeError(error.to_string()),
                Some(&response_text),
            )
        })
    }
}

impl<'de> serde::Deserialize<'de> for UpstreamResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(serde::Deserialize)]
        struct ResponseDeser {
            /// The operation data (if the operation was successful)
            data: Option<Value>,

            /// Any errors that occurred as part of this operation
            errors: Option<Vec<ServerError>>,
        }

        let ResponseDeser { data, errors } = ResponseDeser::deserialize(deserializer)?;

        if data.is_none() && errors.is_none() {
            return Err(D::Error::custom(
                "neither data or errors were present in the upstream response",
            ));
        }

        Ok(UpstreamResponse {
            data: data.unwrap_or(Value::Null),
            errors: errors.unwrap_or_default(),
        })
    }
}

/// If we encountered an error handling a GraphQL response then we want to log the
/// body for debugging purposes.  But we also want to check the HTTP status code
/// and use that as our primary error if it's not success.
fn handle_error_after_response(
    status: StatusCode,
    error: impl Into<Error>,
    #[allow(unused)] response_body: Option<&str>,
) -> Error {
    let error = error.into();
    tracing::debug!("Error in GraphQL connector: {error}");
    if let Some(text) = response_body {
        tracing::debug!("Response Body: {text}");
    }

    if !status.is_success() {
        return Error::HttpErrorResponse(status.as_u16(), response_body.unwrap_or_default().to_string());
    }

    error
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case(200, r#"{"data": {}}"#, UpstreamResponse {
        data: json!({}),
        errors: vec![]
    })]
    #[case(200, r#"{"errors": []}"#, UpstreamResponse {
        data: json!(null),
        errors: vec![]
    })]
    #[case(200, r#"
        {"errors": [{
            "message": "oh no"
        }]}
    "#, UpstreamResponse {
        data: json!(null),
        errors: vec![ServerError {
            message: "oh no".into(),
            source: None,
            locations: vec![],
            path: vec![],
            extensions: None
        }]
    })]
    #[case(500, r#"
        {"errors": [{
            "message": "oh no"
        }]}
    "#, UpstreamResponse {
        data: json!(null),
        errors: vec![ServerError {
            message: "oh no".into(),
            source: None,
            locations: vec![],
            path: vec![],
            extensions: None
        }]
    })]
    #[case(200, r#"{"data": {}}"#, UpstreamResponse {
        data: json!({}),
        errors: vec![]
    })]
    fn test_happy_paths(#[case] status_code: u16, #[case] text: &str, #[case] expected_response: UpstreamResponse) {
        assert_eq!(
            UpstreamResponse::from_response_text(
                StatusCode::from_u16(status_code).unwrap(),
                Ok::<_, Error>(text.to_string())
            )
            .unwrap(),
            expected_response
        );
    }

    #[test]
    fn test_error_paths() {
        let text = r#"{"blah": {}}"#.to_string();
        let response =
            UpstreamResponse::from_response_text(StatusCode::from_u16(500).unwrap(), Ok::<_, Error>(text.clone()))
                .unwrap_err();
        assert_eq!(
            response,
            Error::HttpErrorResponse(500, text),
            "`{response}` does not match"
        );
        let response = UpstreamResponse::from_response_text(
            StatusCode::from_u16(200).unwrap(),
            Ok::<_, Error>(r#"{"blah": {}}"#.into()),
        )
        .unwrap_err();
        assert!(
            matches!(response, Error::JsonDecodeError(_)),
            "{response} does not match"
        );
        let response = UpstreamResponse::from_response_text(
            StatusCode::from_u16(200).unwrap(),
            Ok::<_, Error>(r#"{"errors": "bad"}"#.into()),
        )
        .unwrap_err();
        assert!(
            matches!(response, Error::JsonDecodeError(_)),
            "{response} does not match"
        );
    }
}
