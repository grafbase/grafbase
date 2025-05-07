#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{env, fs, path::Path, process};

const PLUGIN_COMMANDS_PREFIX: &str = "grafbase-";
const PLUGIN_COMMANDS_SUFFIX: &str = env::consts::EXE_SUFFIX;

pub(crate) fn execute(args: &[String]) -> anyhow::Result<()> {
    let Some(name) = args.first() else {
        // it should be unreachable
        return Err(anyhow::anyhow!("No command provided"));
    };

    if name == "gateway" {
        anyhow::bail!(
            "Running grafbase-gateway as a plugin CLI command is not supported. Please run `grafbase-gateway` directly instead of `grafbase gateway`."
        )
    }

    let binary_name = format!("{PLUGIN_COMMANDS_PREFIX}{name}{PLUGIN_COMMANDS_SUFFIX}");

    match which::which(&binary_name) {
        Err(err) => {
            anyhow::bail!(
                "The binary '{binary_name}' is not installed ({err})\nRun `grafbase help` to list built-in commands or `grafbase list-plugins` to list available plugin commands."
            )
        }
        #[cfg(unix)]
        Ok(path) => Err(process::Command::new(path).args(&args[1..]).exec().into()),
        #[cfg(not(unix))]
        Ok(path) => process::Command::new(path)
            .args(&args[1..])
            .output()
            .map(|_| ())
            .map_err(From::from),
    }
}

pub(crate) fn list() -> anyhow::Result<()> {
    let path = path();

    let mut external_commands = Vec::new();

    // Logic inspired by https://github.com/rust-lang/cargo/blob/6cba807e2ce0b7b4bcdf3bdf0a07dcf83988c05a/src/bin/cargo/main.rs#L222
    for dir in std::env::split_paths(&path) {
        let Ok(entries) = fs::read_dir(dir) else { continue };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            let Some(name) = filename
                .strip_prefix(PLUGIN_COMMANDS_PREFIX)
                .and_then(|s| s.strip_suffix(PLUGIN_COMMANDS_SUFFIX))
            else {
                continue;
            };

            if is_executable(entry.path()) && name != "gateway" {
                external_commands.push(name.to_string());
            }
        }
    }

    print_plugin_list(&mut external_commands);

    Ok(())
}

fn print_plugin_list(plugins: &mut Vec<String>) {
    if plugins.is_empty() {
        eprintln!("Found no plugin");
        return;
    }

    // Deduplicate plugins, since they can be in several directories that are in $PATH
    plugins.sort();
    plugins.dedup();

    color_print::cprintln!("<bold><underline>Plugins:</underline></bold>");
    let base_command_name = env::args()
        .next()
        .and_then(|string| {
            Path::new(&string)
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_default();

    for command in plugins {
        println!("  {base_command_name} {command}")
    }
}

fn path() -> String {
    env::var("PATH").unwrap_or_default()
}

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;

    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}
