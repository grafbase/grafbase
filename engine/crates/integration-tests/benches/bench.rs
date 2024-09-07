#![allow(unused_crate_dependencies)]

#[cfg(unix)]
mod federation;

#[cfg(unix)]
criterion::criterion_main!(federation::federation);

#[cfg(not(unix))]
pub fn main() {}
