use extension_catalog::ExtensionCatalog;

use super::sdl::Sdl;

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// A unique identifier of this build of the engine to version cache keys.
/// If built in a git repository, the cache version is taken from the git commit id.
/// For builds outside of a git repository, the build time is used.
fn build_identifier() -> Vec<u8> {
    match built_info::GIT_COMMIT_HASH {
        Some(hash) => hex::decode(hash).expect("Expect hex format"),
        None => built_info::BUILD_TOKEN.as_bytes().to_vec(),
    }
}

pub(super) fn compute(sdl: &Sdl<'_>, extension_catalog: &ExtensionCatalog) -> [u8; 32] {
    let build_id = build_identifier();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&build_id.len().to_ne_bytes());
    hasher.update(&build_id);
    hasher.update(&sdl.raw.len().to_ne_bytes());
    hasher.update(sdl.raw.as_bytes());

    hasher.update(&extension_catalog.len().to_ne_bytes());
    for extension in extension_catalog.iter() {
        serde_json::to_writer(&mut hasher, &extension.manifest).unwrap();
        hasher
            .update_reader(std::fs::File::open(&extension.wasm_path).unwrap())
            .unwrap();
    }

    hasher.finalize().into()
}
