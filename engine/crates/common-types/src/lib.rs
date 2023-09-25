pub mod auth;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum UdfKind {
    Resolver,
    Authorizer,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum OperationType {
    Query { is_introspection: bool },
    Mutation,
    Subscription,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum LogEventType<'a> {
    OperationStarted {
        name: Option<std::borrow::Cow<'a, str>>,
    },
    OperationCompleted {
        name: Option<std::borrow::Cow<'a, str>>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        r#type: OperationType,
    },
    BadRequest {
        name: Option<std::borrow::Cow<'a, str>>,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
    },
    NestedRequest {
        url: String,
        method: String,
        #[serde(rename = "statusCode")]
        status_code: u16,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        duration: std::time::Duration,
        body: Option<String>,
        #[serde(rename = "contentType")]
        content_type: Option<String>,
    },
}
