#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![deny(missing_docs)]

#[cfg(feature = "derive")]
pub use grafbase_hooks_derive::grafbase_hooks;

mod hooks;
pub mod host_io;

pub use hooks::{hooks, HookExports, HookImpls, Hooks};
pub use wit::{
    CacheStatus, Context, EdgeDefinition, Error, ErrorResponse, ExecutedHttpRequest, ExecutedOperation,
    ExecutedSubgraphRequest, FieldError, GraphqlResponseStatus, HeaderError, Headers, LogError, NodeDefinition,
    RequestError, SharedContext, SubgraphRequestExecutionKind, SubgraphResponse,
};

#[doc(hidden)]
pub fn init_hooks(hooks: fn() -> Box<dyn hooks::Hooks>) {
    // SAFETY: This function is called by the gateway at startup, and the hooks are initialized only once. There can
    // be no hook calls during initialization. A hook call is by definition single-threaded.
    unsafe {
        hooks::HOOKS = Some(hooks());
    }
}

/// Registers the hooks type to the gateway. This macro must be called in the library crate root for the local hooks implementation.
#[macro_export]
macro_rules! register_hooks {
    ($name:ident < ($args:tt)* >) => {
        #[doc(hidden)]
        #[export_name = "init-hooks"]
        pub extern "C" fn __init_hooks() -> i64 {
            grafbase_hooks::init_hooks(|| Box::new(<$name<$($args)*> as grafbase_hooks::Hooks>::new()));
            grafbase_hooks::hooks().hook_implementations() as i64
        }

        impl<$($args)*> grafbase_hooks::HookExports for $name<$($args)*> {}
    };
    ($hook_type:ty) => {
        #[doc(hidden)]
        #[export_name = "init-hooks"]
        pub extern "C" fn __init_hooks() -> i64 {
            grafbase_hooks::init_hooks(|| Box::new(<$hook_type as grafbase_hooks::Hooks>::new()));
            grafbase_hooks::hooks().hook_implementations() as i64
        }

        impl grafbase_hooks::HookExports for $hook_type {}
    };
}

mod wit {
    #![allow(clippy::too_many_arguments, clippy::missing_safety_doc, missing_docs)]

    wit_bindgen::generate!({
        skip: ["init-hooks"],
        path: "./wit/world.wit",
    });
}

struct Component;

wit::export!(Component with_types_in wit);
