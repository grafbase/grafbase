use base64::prelude::*;
use std::path::Path;

// Getting a cryptographic hash of the Cargo.lock to ensure we don't mix wasmtime caches.
// Wasmtime seems to have a mechanism for it relying on GIT_REV, but doesn't to work that well.
fn main() -> anyhow::Result<()> {
    let lock_path = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("Cargo.lock");

    let hash = blake3::hash(&std::fs::read(&lock_path).unwrap());
    let mut out = String::new();
    out.push_str("pub const CARGO_LOCK_HASH: &str = \"");
    out.push_str(&BASE64_URL_SAFE_NO_PAD.encode(hash.as_bytes()));
    out.push_str("\";");

    std::fs::write(format!("{}/built.rs", std::env::var("OUT_DIR")?), out)?;

    Ok(())
}
