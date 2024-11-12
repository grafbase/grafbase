use super::ExportersConfig;

use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer};

pub const DEFAULT_SAMPLING: f64 = 0.15;
pub const DEFAULT_COLLECT_VALUE: usize = 128;

/// Tracing configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TracingConfig {
    /// The sampler between 0.0 and 1.0.
    /// Default is 0.15.
    #[serde(deserialize_with = "deserialize_sampling")]
    pub sampling: f64,
    /// Allow clients to specify sampling rate. Enable only if you are not exposing the gateway directly to clients. Default: false.
    pub parent_based_sampler: bool,
    /// Collection configuration
    pub collect: TracingCollectConfig,
    /// Exporters configurations
    pub exporters: ExportersConfig,
    /// Trace parent and context propagation configuration
    pub propagation: PropagationConfig,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            sampling: DEFAULT_SAMPLING,
            collect: Default::default(),
            exporters: Default::default(),
            propagation: Default::default(),
            parent_based_sampler: false,
        }
    }
}

/// Tracing collection configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TracingCollectConfig {
    /// The maximum events per span before discarding.
    /// The default is 128.
    pub max_events_per_span: usize,
    /// The maximum attributes per span before discarding.
    /// The default is 128.
    pub max_attributes_per_span: usize,
    /// The maximum links per span before discarding.
    /// The default is 128.
    pub max_links_per_span: usize,
    /// The maximum attributes per event before discarding.
    /// The default is 128.
    pub max_attributes_per_event: usize,
    /// The maximum attributes per link before discarding.
    /// The default is 128.
    pub max_attributes_per_link: usize,
}

impl Default for TracingCollectConfig {
    fn default() -> Self {
        Self {
            max_events_per_span: DEFAULT_COLLECT_VALUE,
            max_attributes_per_span: DEFAULT_COLLECT_VALUE,
            max_links_per_span: DEFAULT_COLLECT_VALUE,
            max_attributes_per_event: DEFAULT_COLLECT_VALUE,
            max_attributes_per_link: DEFAULT_COLLECT_VALUE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct PropagationConfig {
    /// Enable [TraceContext](https://www.w3.org/TR/trace-context/) propagation through the `traceparent` header. This is the standard trace parent propagation mechanism in OpenTelemetry.
    pub trace_context: bool,
    /// Enable Baggage context propagation through the `baggage` header. This is the standard context propagation mechanism in OpenTelemetry.
    ///
    /// Resources:
    ///
    /// - https://www.w3.org/TR/baggage/
    /// - https://opentelemetry.io/docs/concepts/signals/baggage/
    pub baggage: bool,
    /// Enable AWS X-Ray propagation through the `x-amzn-trace-id` header. This is the standard trace parent propagation mechanism for AWS X-Ray. https://docs.aws.amazon.com/xray/latest/devguide/xray-concepts.html#xray-concepts-tracingheader
    pub aws_xray: bool,
}

fn deserialize_sampling<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let input = f64::deserialize(deserializer)?;

    if !(0.0..=1.0).contains(&input) {
        return Err(DeserializeError::custom("input value should be 0..1"));
    }

    Ok(input)
}
