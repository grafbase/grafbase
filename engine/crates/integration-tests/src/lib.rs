#![allow(unused_crate_dependencies, clippy::panic)]

pub mod engine;
pub mod federation;
pub mod helpers;
pub mod mongodb;
pub mod openid;
pub mod postgres;
pub mod types;
pub mod udfs;

use std::{cell::RefCell, sync::OnceLock};

pub use helpers::{GetPath, ResponseExt};
pub use mongodb::{with_mongodb, with_namespaced_mongodb};
use names::{Generator, Name};
use tokio::runtime::Runtime;
pub use types::{Error, ResponseData};

pub use crate::engine::{Engine, EngineBuilder};

thread_local! {
    static NAMES: RefCell<Option<Generator<'static>>> = RefCell::new(None);
}

#[ctor::ctor]
fn setup_logging() {
    let filter = tracing_subscriber::filter::EnvFilter::try_from_env("RUST_LOG").unwrap_or_default();
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
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

fn random_name() -> String {
    NAMES.with(|maybe_generator| {
        maybe_generator
            .borrow_mut()
            .get_or_insert_with(|| Generator::with_naming(Name::Plain))
            .next()
            .unwrap()
            .replace('-', "")
    })
}
