use schema::{RawInputValue, RawInputValueId, RawInputValues};

pub(crate) type OpInputValues = RawInputValues<Box<str>>;
pub(crate) type OpInputValue = RawInputValue<Box<str>>;
pub(crate) type OpInputValueId = RawInputValueId<Box<str>>;
