#[derive(serde::Deserialize, serde::Serialize)]
pub struct VersionedRegistry<'a> {
    pub registry: std::borrow::Cow<'a, dynaql::registry::Registry>,
    pub deployment_id: std::borrow::Cow<'a, str>, // FIXME: Use `ulid::Ulid` when GB-2683 is fixed.
}
