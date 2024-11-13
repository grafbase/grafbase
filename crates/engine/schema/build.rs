use std::{fmt::Write, process::Command};

use rand::{distributions::Alphanumeric, Rng};

fn main() -> anyhow::Result<()> {
    // Note: the built crate puportes to do this, but it pulls in libgit2
    // for git info which adds minutes onto build time on windows at least,
    // and just seems overkill for what we need
    let mut output = String::new();

    let token = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    // A random token we can use to tell if a build has changed or not.
    writeln!(&mut output, r#"pub const BUILD_TOKEN: &str = "{}";"#, token)?;

    write!(&mut output, r#"pub const GIT_COMMIT_HASH: Option<&str> = "#)?;

    match get_current_git_hash() {
        Some(git_hash) => {
            writeln!(&mut output, r#"Some("{}");"#, git_hash)?;
        }
        None => {
            writeln!(&mut output, "None;")?;
        }
    }

    std::fs::write(format!("{}/built.rs", std::env::var("OUT_DIR")?), output)?;

    Ok(())
}

fn get_current_git_hash() -> Option<String> {
    let output = Command::new("git")
        .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .args(["rev-parse", "HEAD"])
        .output()
        .inspect_err(|e| eprintln!("Could not run git: {e}"))
        .ok()?;

    if !output.status.success() {
        eprintln!("Git returned status {}", output.status);
        return None;
    }

    Some(std::str::from_utf8(&output.stdout).unwrap().trim().to_string())
}
