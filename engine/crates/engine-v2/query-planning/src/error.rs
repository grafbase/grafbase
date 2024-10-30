#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Requirement cycle detected")]
    RequirementCycleDetected,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
