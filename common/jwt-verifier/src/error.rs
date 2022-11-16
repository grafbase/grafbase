use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum VerificationError {
        HttpRequest(err: surf::Error) {
            display("{err}")
        }
        Integrity(err: jwt_compact::ValidationError) {
            display("{err}")
        }
        InvalidIssuerUrl {
            display("issuer URL mismatch")
        }
        InvalidIssueTime {
            display("invalid issue time")
        }
        InvalidGroups(claim: String) {
            display("invalid groups claim {claim:?}")
        }
        UnsupportedAlgorithm(algo: String) {
            display("unsupported algorithm: {algo}")
        }
        InvalidToken {
            display("invalid OIDC token")
        }
        JwkNotFound(kid: String) {
            display("no JWK found to verify tokens with kid {kid}")
        }
        JwkFormat {
            display("invalid JWK format")
        }
        CacheError(err: worker::kv::KvError) {
            display("{err}")
        }
    }
}
