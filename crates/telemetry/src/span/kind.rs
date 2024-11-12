/// Used to filter spans in ClickHouse and the API. The span name is most of the time dynamically
/// computed, so we add a `grafbase.kind` attribute.
#[derive(Debug, strum::Display, strum::AsRefStr, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[non_exhaustive]
pub(crate) enum GrafbaseSpanKind {
    HttpRequest,
    GraphqlOperation,
    SubgraphGraphqlRequest,
}
