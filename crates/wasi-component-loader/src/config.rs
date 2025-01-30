use gateway_config::WasiExtensionsConfig;
pub use gateway_config::{extensions::ExtensionsConfig, hooks::HooksWasiConfig};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder};

pub(crate) fn build_extensions_context(config: &WasiExtensionsConfig) -> WasiCtx {
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

pub(crate) fn build_hooks_context(
    HooksWasiConfig {
        max_pool_size: _,
        location: _,
        networking,
        environment_variables,
        stdout,
        stderr,
        preopened_directories,
    }: &HooksWasiConfig,
) -> WasiCtx {
    let mut builder = WasiCtxBuilder::new();

    if *networking {
        builder.inherit_network();
        builder.allow_tcp(true);
        builder.allow_udp(true);
        builder.allow_ip_name_lookup(true);
    }

    if *environment_variables {
        builder.inherit_env();
    }

    if *stdout {
        builder.inherit_stdout();
    }

    if *stderr {
        builder.inherit_stderr();
    }

    for dir in preopened_directories {
        let mut dir_permissions = DirPerms::empty();
        let mut file_permissions = FilePerms::empty();

        if dir.read_permission {
            dir_permissions = dir_permissions.union(DirPerms::READ);
            file_permissions = file_permissions.union(FilePerms::READ);
        }

        if dir.write_permission {
            dir_permissions = dir_permissions.union(DirPerms::MUTATE);
            file_permissions = file_permissions.union(FilePerms::WRITE);
        }

        builder
            .preopened_dir(&dir.host_path, &dir.guest_path, dir_permissions, file_permissions)
            .unwrap();
    }

    builder.build()
}
