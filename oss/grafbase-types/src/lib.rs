pub mod auth;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize, strum::Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum UdfKind {
    Resolver,
    Authorizer,
}
