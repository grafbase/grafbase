mod pool;

pub(crate) use pool::*;

/// Until engine-v2 is entirely remove
#[cfg(feature = "tokio")]
pub(crate) fn block_in_place<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    tokio::task::block_in_place(f)
}

#[cfg(not(feature = "tokio"))]
pub(crate) fn block_in_place<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}
