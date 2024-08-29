use std::{env, path};

pub fn cargo_bin<S: AsRef<str>>(name: S) -> path::PathBuf {
    cargo_bin_str(name.as_ref())
}

fn cargo_bin_str(name: &str) -> path::PathBuf {
    let env_var = format!("CARGO_BIN_EXE_{name}");
    std::env::var_os(env_var).map_or_else(
        || target_dir().join(format!("{name}{}", env::consts::EXE_SUFFIX)),
        std::convert::Into::into,
    )
}

fn target_dir() -> path::PathBuf {
    env::current_exe()
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
