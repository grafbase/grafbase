#![allow(dead_code)]
use futures::Future;
#[cfg(feature = "wasm_runtime")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug)]
pub(crate) enum Runtime {
    #[cfg(feature = "tokio_runtime")]
    Tokio,
    #[cfg(feature = "wasm_runtime")]
    Wasm,
}

impl Runtime {
    pub const fn locate() -> Self {
        #[cfg(all(feature = "tokio_runtime", not(feature = "wasm_runtime")))]
        {
            Runtime::Tokio
        }

        #[cfg(all(not(feature = "tokio_runtime"), feature = "wasm_runtime"))]
        {
            Runtime::Wasm
        }

        #[cfg(all(feature = "tokio_runtime", feature = "wasm_runtime"))]
        {
            Runtime::Tokio
        }

        #[cfg(all(not(feature = "tokio_runtime"), not(feature = "wasm_runtime")))]
        {
            compile_error!("tokio_runtime or wasm_runtime features required")
        }
    }

    #[allow(dead_code)]
    pub fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        match self {
            #[cfg(feature = "tokio_runtime")]
            Runtime::Tokio => {
                tokio::spawn(f);
            }
            #[cfg(feature = "wasm_runtime")]
            Runtime::Wasm => {
                spawn_local(f);
            }
        };
    }
}
