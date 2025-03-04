mod v1;

pub use self::v1::*;

pub const EXTENSION_LOCKFILE_NAME: &str = "grafbase-extensions.lock";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(tag = "version")]
pub enum VersionedLockfile {
    #[serde(rename = "1")]
    V1(v1::Lockfile),
}

impl Default for VersionedLockfile {
    fn default() -> Self {
        VersionedLockfile::V1(v1::Lockfile::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_lockfile() {
        let lockfile = VersionedLockfile::V1(v1::Lockfile::default());

        let json = serde_json::to_string(&lockfile).unwrap();
        assert_eq!(json, "{\"version\":\"1\",\"extensions\":[]}");
    }
}
