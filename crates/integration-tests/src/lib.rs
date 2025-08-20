#![allow(clippy::panic)]

pub mod fetch;
pub mod gateway;
pub mod types;

mod mock_trusted_documents;

use std::sync::{LazyLock, OnceLock};

pub use mock_trusted_documents::TestTrustedDocument;
use regex::{Captures, Regex};
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

pub fn cleanup_error(err: impl std::fmt::Display) -> String {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"file:///tmp/.*/extensions").expect("Failed to compile regex for file URLs"));
    RE.replace_all(&err.to_string(), |caps: &Captures<'_>| {
        let n = caps[0].len();
        format!("file:///tmp/{}/extensions", "X".repeat(n - 23))
    })
    .to_string()
}
