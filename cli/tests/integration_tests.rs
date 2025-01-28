use std::{
    env::{consts::EXE_SUFFIX, current_exe},
    path::PathBuf,
};

mod extension;

pub fn cargo_bin<S: AsRef<str>>(name: S) -> PathBuf {
    cargo_bin_str(name.as_ref())
}

fn target_dir() -> PathBuf {
    current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .unwrap()
}

fn cargo_bin_str(name: &str) -> PathBuf {
    let env_var = format!("CARGO_BIN_EXE_{name}");
    std::env::var_os(env_var).map_or_else(
        || target_dir().join(format!("{name}{}", EXE_SUFFIX)),
        std::convert::Into::into,
    )
}
