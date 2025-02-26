use std::{str::FromStr, time::Duration};

use grafbase_sdk::host_io::pubsub::nats::{self, OffsetDateTime};

#[derive(Debug)]
pub enum DirectiveKind {
    Publish,
    Request,
}

impl FromStr for DirectiveKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "natsPublish" => Ok(DirectiveKind::Publish),
            "natsRequest" => Ok(DirectiveKind::Request),
            _ => Err(format!("Unknown directive: {}", s)),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishArguments<'a> {
    pub provider: &'a str,
    pub subject: &'a str,
    body: Option<Body>,
}

impl PublishArguments<'_> {
    pub fn body(&self) -> Option<&serde_json::Value> {
        self.body.as_ref().and_then(|body| {
            body.r#static
                .as_ref()
                .or_else(|| body.selection.as_ref().and_then(|s| s.input.as_ref()))
        })
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestArguments<'a> {
    pub provider: &'a str,
    pub subject: &'a str,
    pub selection: Option<&'a str>,
    #[serde(rename = "timeoutMs", deserialize_with = "deserialize_duration_from_ms")]
    pub timeout: Duration,
    body: Option<Body>,
}

impl RequestArguments<'_> {
    pub fn body(&self) -> Option<&serde_json::Value> {
        self.body.as_ref().and_then(|body| {
            body.r#static
                .as_ref()
                .or_else(|| body.selection.as_ref().and_then(|s| s.input.as_ref()))
        })
    }
}

fn deserialize_duration_from_ms<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let ms = u64::deserialize(deserializer)?;

    Ok(Duration::from_millis(ms))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub selection: Option<RestInput>,
    pub r#static: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestInput {
    input: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NatsPublishResult {
    pub success: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeArguments<'a> {
    pub provider: &'a str,
    pub subject: &'a str,
    pub selection: Option<String>,
    pub stream_config: Option<NatsStreamConfiguration<'a>>,
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatsStreamConfiguration<'a> {
    pub stream_name: &'a str,
    pub consumer_name: &'a str,
    pub durable_name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub inactive_threshold_ms: u64,
    deliver_policy: NatsStreamDeliverPolicy,
}

impl NatsStreamConfiguration<'_> {
    pub fn deliver_policy(self) -> nats::NatsStreamDeliverPolicy {
        match self.deliver_policy.r#type {
            NatsStreamDeliverPolicyType::All => nats::NatsStreamDeliverPolicy::All,
            NatsStreamDeliverPolicyType::Last => nats::NatsStreamDeliverPolicy::Last,
            NatsStreamDeliverPolicyType::New => nats::NatsStreamDeliverPolicy::New,
            NatsStreamDeliverPolicyType::ByStartSequence => {
                nats::NatsStreamDeliverPolicy::ByStartSequence(self.deliver_policy.start_sequence.unwrap_or(0))
            }
            NatsStreamDeliverPolicyType::ByStartTime => {
                let time = match self.deliver_policy.start_time_ms {
                    Some(ms) => OffsetDateTime::from_unix_timestamp_nanos((ms as i128) * 1_000_000).unwrap(),
                    None => OffsetDateTime::now_utc(),
                };

                nats::NatsStreamDeliverPolicy::ByStartTime(time)
            }
            NatsStreamDeliverPolicyType::LastPerSubject => nats::NatsStreamDeliverPolicy::LastPerSubject,
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatsStreamDeliverPolicy {
    r#type: NatsStreamDeliverPolicyType,
    start_sequence: Option<u64>,
    start_time_ms: Option<u64>,
}

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NatsStreamDeliverPolicyType {
    All,
    Last,
    New,
    ByStartSequence,
    ByStartTime,
    LastPerSubject,
}
