use gateway_integration_tests::clickhouse_client;
use indoc::formatdoc;

use crate::{
    load_schema,
    telemetry::metrics::{ExponentialHistogramRow, METRICS_DELAY},
    with_hybrid_server,
};

#[test]
fn gdn_update() {
    let service_name = format!("service_{}", ulid::Ulid::new());
    let schema = load_schema("big");
    let clickhouse = clickhouse_client();

    let config = &formatdoc! {r#"
        [graph]
        introspection = true

        [telemetry]
        service_name = "{service_name}"

        [telemetry.tracing]
        sampling = 1

        [telemetry.exporters.otlp]
        enabled = true
        endpoint = "http://localhost:4318"
        protocol = "grpc"

        [telemetry.exporters.otlp.batch_export]
        scheduled_delay = 1
        max_export_batch_size = 1
    "#};

    with_hybrid_server(config, "test_graph", &schema, |client, _, addr| async move {
        let resp = client.execute("query SimpleQuery { __typename }").await.into_body();

        insta::assert_json_snapshot!(resp, @r###"
            {
              "data": {
                "__typename": "Query"
              }
            }
            "###);

        tokio::time::sleep(METRICS_DELAY).await;

        let query = indoc::indoc! {r#"
            SELECT Count, Attributes
            FROM otel_metrics_exponential_histogram
            WHERE ServiceName = ?
                AND ScopeName = 'grafbase'
                AND MetricName = 'gdn.request.duration'
        "#};

        let row = clickhouse
            .query(query)
            .bind(&service_name)
            .fetch_optional::<ExponentialHistogramRow>()
            .await
            .unwrap()
            .unwrap();

        assert_eq!(1, row.count);
        assert_eq!(Some("NEW"), row.attributes.get("gdn.response.kind").map(|s| s.as_str()),);

        assert_eq!(
            Some(&format!("http://{addr}/graphs/test_graph/current")),
            row.attributes.get("server.address"),
        );
    });
}
