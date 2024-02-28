use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    certificate: PathBuf,
    key: PathBuf,
}

impl TlsConfig {
    pub fn certificate(&self) -> &Path {
        self.certificate.as_path()
    }

    pub fn key(&self) -> &Path {
        self.key.as_path()
    }
}
