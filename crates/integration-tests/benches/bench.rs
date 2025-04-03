use mimalloc::MiMalloc;

#[cfg(unix)]
mod gateway;

#[cfg(unix)]
criterion::criterion_main!(gateway::federation);

#[cfg(not(unix))]
pub fn main() {}

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
