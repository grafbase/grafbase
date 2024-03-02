#[derive(Debug, thiserror::Error)]
#[error("{:?}", self)]
pub enum AdminError {
    #[error("Error purging cache - {0}")]
    CachePurgeError(String),
}
