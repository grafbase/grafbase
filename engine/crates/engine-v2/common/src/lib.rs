mod request;
mod response;

pub use request::*;
pub use response::*;

pub(crate) mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));

    pub(crate) fn git_version() -> &'static [u8; 20] {
        GIT_COMMIT_HASH
            .expect("missing git version")
            .as_bytes()
            .try_into()
            .expect("git commit hash is 20 bytes")
    }
}
