use mimalloc::MiMalloc;

#[cfg(unix)]
mod federation;

#[cfg(unix)]
criterion::criterion_main!(federation::federation);

#[cfg(not(unix))]
pub fn main() {}

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
