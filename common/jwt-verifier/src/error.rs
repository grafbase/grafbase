#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("{0}")]
    HttpRequest(reqwest::Error),
    #[error("{0}")]
    Integrity(jwt_compact::ValidationError),
    #[error("issuer claim mismatch")]
    IssuerClaimMismatch,
    #[error("invalid issue time")]
    InvalidIssueTime,
    #[error("audience does not match client ID")]
    InvalidAudience,
    #[error("invalid groups claim {claim}")]
    InvalidGroups { claim: String },
    #[error("unsupported algorithm: {algorithm}")]
    UnsupportedAlgorithm { algorithm: String },
    #[error("invalid OIDC token")]
    InvalidToken,
    #[error("no JWK found to verify tokens with kid {kid}")]
    JwkNotFound { kid: String },
    #[error("invalid JWK format")]
    JwkFormat,
    #[error("{0}")]
    IssuerFormat(url::ParseError),
}
