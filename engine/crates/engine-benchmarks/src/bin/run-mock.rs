#![allow(unused_crate_dependencies)]

use graphql_mocks::{LargeResponseSchema, MockGraphQlServer};

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // Could at some point make this take CLI args
    let mock = runtime.block_on(MockGraphQlServer::new(LargeResponseSchema));

    println!(r#"{{"port": {}}}"#, mock.port());

    // This is designed to be SIGKILLed so just block indetinitely
    runtime.block_on(mock.block())
}
