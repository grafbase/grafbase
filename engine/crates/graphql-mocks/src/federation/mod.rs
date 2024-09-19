// Mostly taken from:
// https://github.com/async-graphql/examples
mod accounts;
mod inventory;
mod products;
mod reviews;
mod secure;
mod shipping;

pub use self::shipping::FederatedShippingSchema;
pub use accounts::FederatedAccountsSchema;
pub use inventory::FederatedInventorySchema;
pub use products::FederatedProductsSchema;
pub use reviews::FederatedReviewsSchema;
pub use secure::SecureFederatedSchema;
