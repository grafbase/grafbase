#[cfg(feature = "lambda")]
mod lambda;
mod log;
#[cfg(not(feature = "lambda"))]
mod std;

pub(crate) use log::LogLevel;

#[cfg(feature = "lambda")]
pub(crate) use lambda::Args;

#[cfg(not(feature = "lambda"))]
pub(crate) use self::std::Args;
