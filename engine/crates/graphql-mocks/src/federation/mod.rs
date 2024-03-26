// Mostly taken from:
// https://github.com/async-graphql/examples
mod accounts;
mod inventory;
mod products;
mod reviews;

pub use accounts::FakeFederationAccountsSchema;
pub use inventory::FakeFederationInventorySchema;
pub use products::FakeFederationProductsSchema;
pub use reviews::FakeFederationReviewsSchema;
