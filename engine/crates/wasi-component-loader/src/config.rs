pub use gateway_config::hooks::HooksWasiConfig as Config;
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder};

pub(crate) fn build_wasi_context(config: &Config) -> WasiCtx {
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

    for dir in &config.preopened_directories {
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
