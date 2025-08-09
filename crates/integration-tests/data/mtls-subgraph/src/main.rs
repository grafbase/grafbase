use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::post, Router};
use axum_server::tls_rustls::RustlsConfig;
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
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

    let handle = axum_server::Handle::new();

    // Spawn a task to gracefully shutdown server.
    tokio::spawn(graceful_shutdown(handle.clone()));

    // Start the server with TLS
    axum_server::bind_rustls(addr, tls_config)
        .handle(handle)
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

    // Collect certificates into a Vec and add them to the root store
    let ca_certs = certs(&mut ca_file).collect::<Result<Vec<_>, _>>()?;

    for cert in ca_certs {
        root_store
            .add(cert)
            .map_err(|e| format!("Failed to add CA cert: {:?}", e))?;
    }

    // Setup client certificate verifier that requires clients to authenticate
    let client_verifier = WebPkiClientVerifier::builder(root_store.into())
        .build()
        .map_err(|e| format!("Failed to build client verifier: {:?}", e))?;

    // Load server certificates
    let mut cert_file = BufReader::new(File::open("certs/server-cert.pem")?);
    let server_certs = certs(&mut cert_file).collect::<Result<Vec<_>, _>>()?;

    // Load server private key
    let mut key_file = BufReader::new(File::open("certs/server-key.pem")?);
    let keys = pkcs8_private_keys(&mut key_file).collect::<Result<Vec<_>, _>>()?;
    let key = keys.into_iter().next().ok_or("No private key found")?;

    // Create server config with client certificate verification
    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(server_certs, key.into())
        .map_err(|e| format!("TLS error: {:?}", e))?;

    Ok(RustlsConfig::from_config(Arc::new(config)))
}

async fn graceful_shutdown(handle: axum_server::Handle) {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutting down gracefully...");
    handle.graceful_shutdown(Some(std::time::Duration::from_secs(3)));
}
