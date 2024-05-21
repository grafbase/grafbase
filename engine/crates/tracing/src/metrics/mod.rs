mod operation;
mod request;

use std::borrow::Cow;

use opentelemetry::metrics::{Meter, MeterProvider};
pub use operation::*;
pub use request::*;

const SCOPE: &str = "grafbase";
const SCOPE_VERSION: &str = "1.0";

pub fn meter_from_global_provider() -> Meter {
    meter(&opentelemetry::global::meter_provider())
}

pub fn meter(provider: &impl MeterProvider) -> Meter {
    provider.versioned_meter(SCOPE, Some(SCOPE_VERSION), None::<Cow<'static, str>>, None)
}
