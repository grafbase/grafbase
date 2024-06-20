use std::path::{Path, PathBuf};

use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder};

/// GraphQL WASI component configuration.
#[derive(Default, Debug, serde::Deserialize)]
pub struct Config {
    location: PathBuf,
    #[serde(default)]
    networking: bool,
    #[serde(default)]
    environment_variables: bool,
    #[serde(default)]
    stdout: bool,
    #[serde(default)]
    stderr: bool,
    #[serde(default)]
    preopened_directories: Vec<PreopenedDirectory>,
}

/// Configuration for allowing access to a certain directory from a WASI guest
#[derive(Debug, serde::Deserialize)]
pub struct PreopenedDirectory {
    host_path: PathBuf,
    guest_path: String,
    read_permission: bool,
    write_permission: bool,
}

impl Config {
    pub(crate) fn location(&self) -> &Path {
        &self.location
    }

    pub(crate) fn networking_enabled(&self) -> bool {
        self.networking
    }

    pub(crate) fn wasi_context(&self) -> WasiCtx {
        let mut builder = WasiCtxBuilder::new();

        if self.networking {
            builder.inherit_network();
            builder.allow_tcp(true);
            builder.allow_udp(true);
            builder.allow_ip_name_lookup(true);
        }

        if self.environment_variables {
            builder.inherit_env();
        }

        if self.stdout {
            builder.inherit_stdout();
        }

        if self.stderr {
            builder.inherit_stderr();
        }

        for dir in &self.preopened_directories {
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
}
