#![allow(clippy::panic)]

pub mod fetch;
pub mod gateway;
pub mod openid;
pub mod types;

mod mock_trusted_documents;

use std::sync::OnceLock;

pub use mock_trusted_documents::TestTrustedDocument;
use tokio::runtime::Runtime;
pub use types::{Error, ResponseData};

#[ctor::ctor]
fn setup_rustls() {
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();
}

#[ctor::ctor]
fn setup_logging() {
    cfg_if::cfg_if! {
        // avoids tracing in benchmarks
        if #[cfg(test)] {
            let filter =  tracing_subscriber::filter::EnvFilter::builder()
                .parse(std::env::var("RUST_LOG").unwrap_or("engine=debug".to_string()))
                .unwrap();
        } else {
            let filter = tracing_subscriber::filter::EnvFilter::from_default_env();
        }
    }
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .without_time()
        .init();
}

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
    })
}
