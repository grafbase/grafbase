mod chars_max_length;
mod chars_min_length;
mod email;
mod ip;
mod max_items;
mod max_length;
mod maximum;
mod min_items;
mod min_length;
mod minimum;
mod multiple_of;
mod regex;
mod url;

pub use chars_max_length::chars_max_length;
pub use chars_min_length::chars_min_length;
pub use email::email;
pub use ip::ip;
pub use max_items::max_items;
pub use max_length::max_length;
pub use maximum::maximum;
pub use min_items::min_items;
pub use min_length::min_length;
pub use minimum::minimum;
pub use multiple_of::multiple_of;

pub use self::{regex::regex, url::url};
use crate::LegacyInputType;

/// Represents a custom input value validator.
pub trait CustomValidator<T: LegacyInputType> {
    /// Check the value is valid.
    fn check(&self, value: &T) -> Result<(), String>;
}
