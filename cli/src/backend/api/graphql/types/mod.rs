cynic::impl_scalar!(chrono::DateTime<chrono::Utc>, schema::DateTime);
cynic::impl_scalar!(semver::VersionReq, schema::SemverVersionRequirement);

pub(crate) mod mutations;
pub(crate) mod queries;

#[cynic::schema("grafbase")]
mod schema {}
