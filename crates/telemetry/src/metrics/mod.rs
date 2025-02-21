mod engine;
mod request;

pub use engine::*;
use opentelemetry::{
    InstrumentationScope,
    metrics::{Meter, MeterProvider},
};
pub use request::*;

pub fn meter_from_global_provider() -> Meter {
    meter(&*opentelemetry::global::meter_provider())
}

pub fn meter(provider: &dyn MeterProvider) -> Meter {
    let scope = InstrumentationScope::builder(crate::SCOPE)
        .with_version(crate::SCOPE_VERSION)
        .build();

    provider.meter_with_scope(scope)
}
