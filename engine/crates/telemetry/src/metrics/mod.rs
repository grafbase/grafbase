mod engine;
mod request;

pub use engine::*;
use opentelemetry::metrics::{Meter, MeterProvider};
pub use request::*;

pub fn meter_from_global_provider() -> Meter {
    meter(&*opentelemetry::global::meter_provider())
}

pub fn meter(provider: &dyn MeterProvider) -> Meter {
    provider.versioned_meter(crate::SCOPE, Some(crate::SCOPE_VERSION), None::<&'static str>, None)
}
