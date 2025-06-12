use wasmtime_wasi::{p2::WasiCtx, p2::WasiCtxBuilder};

use crate::extension::WasmConfig;

pub(crate) fn build_context(config: &WasmConfig) -> WasiCtx {
    let mut builder = WasiCtxBuilder::new();

    if config.networking {
        builder.inherit_network();
        builder.allow_tcp(true);
        builder.allow_udp(true);
        builder.allow_ip_name_lookup(true);
    }

    if config.environment_variables {
        builder.inherit_env();
    }

    if config.stdout {
        builder.inherit_stdout();
    }

    if config.stderr {
        builder.inherit_stderr();
    }

    builder.build()
}
