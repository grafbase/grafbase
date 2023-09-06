use rusoto_core::RusotoError;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "__type")]
enum AwsError {
    #[serde(rename = "com.amazon.coral.service#UnrecognizedClientException")]
    UnrecognizedClientException { message: String },
}

/// This does not replace RetryingDynamodb. Unfortunately we cannot extend the latter to support
/// this special case we have as we're creating users on the fly for AWS.
pub(crate) async fn rusoto_retry<E, T>(task: T) -> Result<T::Item, T::Error>
where
    T: again::Task<Error = RusotoError<E>>,
{
    again::RetryPolicy::exponential(std::time::Duration::from_millis(200))
        .with_max_retries(2)
        .retry_if(task, |err: &RusotoError<E>| match err {
            // This usually happens when credentials weren't propagated in AWS yet for the newly
            // created user, which happens regularly in e2e-tests.
            RusotoError::Unknown(response) if response.status.as_u16() == 400 => {
                serde_json::from_slice::<AwsError>(&response.body)
                    .map(|err| match err {
                        AwsError::UnrecognizedClientException { message } => {
                            message == "The security token included in the request is invalid."
                        }
                    })
                    .unwrap_or_default()
            }
            _ => false,
        })
        .await
}
