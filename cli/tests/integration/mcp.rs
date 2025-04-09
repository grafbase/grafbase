use duct::cmd;
use graphql_mocks::Subgraph as _;
use rand::random;

use crate::cargo_bin;

#[tokio::test]
async fn test_mcp() {
    let subgraph = graphql_mocks::EchoSchema.start().await;

    // Pick a port number in the dynamic range.
    let port = random::<u16>() | 0xc000;

    let _handle = cmd(
        cargo_bin("grafbase"),
        &["mcp", subgraph.url().as_str(), "--port", &port.to_string()],
    )
    .unchecked()
    .start()
    .unwrap();

    for _ in 0..40 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let Ok(response) = reqwest::Client::new()
            .get(format!("http://127.0.0.1:{port}/mcp").parse::<url::Url>().unwrap())
            .header(http::header::ACCEPT, "text/event-stream")
            .send()
            .await
        else {
            continue;
        };
        if response.status() == 200 {
            return;
        }
    }

    panic!("Failed to connect to MCP server");
}
