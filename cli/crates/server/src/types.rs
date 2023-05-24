use common::types::ResolverMessageLevel;
use std::path::PathBuf;

pub const ASSETS_GZIP: &[u8] = include_bytes!("../assets/assets.tar.gz");

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
