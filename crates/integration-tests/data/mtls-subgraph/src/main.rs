use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::post, Router};
use axum_server::tls_rustls::RustlsConfig;
use rustls::server::AllowAnyAuthenticatedClient;
use rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

// Define the GraphQL schema root query type
#[derive(Default)]
struct Query;

#[async_graphql::Object]
impl Query {
    /// Returns a hello world message
    async fn hello(&self) -> &str {
        "Hello, world"
    }
}

// Create our GraphQL schema
type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the GraphQL schema
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    // Create an Axum router with our GraphQL endpoint
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .with_state(schema);

    // Setup TLS config for the server with mTLS
    let tls_config = configure_tls()?;

    // Define the address to listen on
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("GraphQL server with mTLS starting on https://{}/graphql", addr);

    // Start the server with TLS
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// GraphQL handler for Axum
async fn graphql_handler(schema: axum::extract::State<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// Configure TLS with mutual authentication (mTLS)
fn configure_tls() -> Result<RustlsConfig, Box<dyn std::error::Error>> {
    // Load CA certificate for client verification
    let mut root_store = RootCertStore::empty();
    let mut ca_file = BufReader::new(File::open("certs/ca-cert.pem")?);
    for cert in certs(&mut ca_file)? {
        root_store
            .add(&Certificate(cert))
            .map_err(|e| format!("Failed to add CA cert: {:?}", e))?;
    }

    // Setup client certificate verifier that requires clients to authenticate
    let client_auth = AllowAnyAuthenticatedClient::new(root_store);

    // Load server certificates
    let mut cert_file = BufReader::new(File::open("certs/server-cert.pem")?);
    let server_certs = certs(&mut cert_file)?.into_iter().map(Certificate).collect();

    // Load server private key
    let mut key_file = BufReader::new(File::open("certs/server-key.pem")?);
    let mut keys = pkcs8_private_keys(&mut key_file)?;
    if keys.is_empty() {
        return Err("No private key found".into());
    }

    // Create server config with client certificate verification
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(client_auth)
        .with_single_cert(server_certs, PrivateKey(keys.remove(0)))
        .map_err(|e| format!("TLS error: {:?}", e))?;

    Ok(RustlsConfig::from_config(Arc::new(config)))
}
