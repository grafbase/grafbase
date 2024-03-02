pub mod v1;
pub mod v2;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum VersionedAuthConfig {
    V1(v1::AuthConfig),
    V2(v2::AuthConfig),
}
