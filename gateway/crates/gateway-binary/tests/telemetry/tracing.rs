use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use futures_util::Future;
use indoc::formatdoc;

use crate::{clickhouse_client, runtime, Client};

#[test]
fn propagation() {
    with_mock_subgraph(
        "",
        graphql_mocks::EchoSchema,
        |service_name, start, gateway, clickhouse| async move {
            let request = r#"
                query {
                    headers {
                        name
                        value
                    }
                }
            "#;

            let response: serde_json::Value = gateway
                .gql(request)
                .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
                .send()
                .await;

            panic!("response: {:#?}", response);
        },
    );
}

fn with_mock_subgraph<T, F>(config: &str, subgraph_schema: impl graphql_mocks::Schema + 'static, test: T)
where
    T: FnOnce(String, u64, Arc<Client>, &'static clickhouse::Client) -> F,
    F: Future<Output = ()>,
{
    let service_name = format!("service_{}", ulid::Ulid::new());
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

        {config}
    "#};

    let clickhouse = clickhouse_client();

    println!("service_name: {service_name}");
    println!("{config}");

    let subgraph_sdl = subgraph_schema.sdl();

    let subgraph_server = runtime().block_on(async { graphql_mocks::MockGraphQlServer::new(subgraph_schema).await });

    let federated_schema = {
        let parsed = async_graphql_parser::parse_schema(&subgraph_sdl).unwrap();
        let mut subgraphs = graphql_composition::Subgraphs::default();
        subgraphs.ingest(&parsed, "the-subgraph", subgraph_server.url().as_str());
        graphql_composition::compose(&subgraphs)
            .into_result()
            .unwrap()
            .into_sdl()
            .unwrap()
    };

    super::with_static_server(config, &federated_schema, None, None, |client| async move {
        const WAIT_SECONDS: u64 = 2;
        let start = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + WAIT_SECONDS;

        // wait for initial polling to be pushed to OTEL tables so we can ignore it with the
        // appropriate start time filter.
        tokio::time::sleep(Duration::from_secs(WAIT_SECONDS)).await;

        test(service_name, start, client, clickhouse).await
    })
}
