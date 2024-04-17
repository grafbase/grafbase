mod operation;
mod request;

use opentelemetry::metrics::{Meter, MeterProvider};
pub use operation::*;
pub use request::*;

const SCOPE: &str = "grafbase";

pub fn meter_from_global_provider() -> Meter {
    meter(opentelemetry::global::meter_provider())
}

pub fn meter(provider: impl MeterProvider) -> Meter {
    provider.meter(SCOPE)
}
