#![allow(unused_crate_dependencies, clippy::panic)]

pub mod engine;
pub mod federation;
pub mod helpers;
pub mod mocks;
pub mod mongodb;
pub mod postgres;
pub mod types;
pub mod udfs;

use std::{cell::RefCell, sync::OnceLock};

pub use helpers::{GetPath, ResponseExt};
pub use mocks::graphql::MockGraphQlServer;
pub use mongodb::{with_mongodb, with_namespaced_mongodb};
use names::{Generator, Name};
use tokio::runtime::Runtime;
pub use types::{Error, ResponseData};

pub use crate::engine::{Engine, EngineBuilder};

thread_local! {
    static NAMES: RefCell<Option<Generator<'static>>> = RefCell::new(None);
}

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
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
