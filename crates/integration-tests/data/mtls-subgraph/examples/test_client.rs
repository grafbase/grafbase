use reqwest::ClientBuilder;
use serde_json::{Value, json};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_certificate = reqwest::Certificate::from_pem(&std::fs::read("certs/ca-cert.pem")?)?;
    let identity = reqwest::Identity::from_pem(&std::fs::read("certs/client-identity.pem")?)?;

    // Create the HTTPS client
    let client = ClientBuilder::new()
        .use_rustls_tls()
        .tls_built_in_root_certs(false)
        .add_root_certificate(root_certificate)
        .identity(identity)
        .https_only(true)
        .build()?;

    // GraphQL query
    let query = json!({
        "query": "{ hello }"
    });

    println!("Sending GraphQL request to the server...");

    // Send GraphQL request
    let response = client
        .post("https://localhost:8081/graphql")
        .json(&query)
        .send()
        .await?;

    // Check response status
    println!("Response status: {}", response.status());

    // Parse and print response
    let response_body: Value = response.json().await?;
    println!("Response: {}", serde_json::to_string_pretty(&response_body)?);

    Ok(())
}
