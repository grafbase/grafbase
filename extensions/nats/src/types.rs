use std::str::FromStr;

#[derive(Debug)]
pub enum DirectiveKind {
    Publish,
}

impl FromStr for DirectiveKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "natsPublish" => Ok(DirectiveKind::Publish),
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
    #[allow(dead_code)] // will get to this with jetstream
    pub stream_config: Option<NatsStreamConfiguration<'a>>,
}

#[allow(dead_code)] // will get to this with jetstream
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatsStreamConfiguration<'a> {
    pub consumer: &'a str,
    pub stream: &'a str,
}
