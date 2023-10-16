cynic::impl_scalar!(chrono::DateTime<chrono::Utc>, schema::DateTime);

pub mod mutations;
pub mod queries;

#[cynic::schema("grafbase")]
mod schema {}
