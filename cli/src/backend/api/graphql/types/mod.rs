cynic::impl_scalar!(chrono::DateTime<chrono::Utc>, schema::DateTime);

pub(crate) mod mutations;
pub(crate) mod queries;

#[cynic::schema("grafbase")]
mod schema {}
