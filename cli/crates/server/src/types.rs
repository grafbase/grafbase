use common::types::ResolverMessageLevel;
use rust_embed::RustEmbed;
use std::path::PathBuf;

pub const MY_DATA: &[u8] = include_bytes!("../assets.tar.gz");

// #[derive(RustEmbed)]
// #[folder = "assets/"]
pub struct Assets;

#[derive(Clone, Debug)]
pub enum ServerMessage {
    Ready(u16),
    Reload(PathBuf),
    StartResolverBuild(String),
    CompleteResolverBuild {
        name: String,
        duration: std::time::Duration,
    },
    ResolverMessage {
        resolver_name: String,
        level: ResolverMessageLevel,
        message: String,
    },
    CompilationError(String),
}
