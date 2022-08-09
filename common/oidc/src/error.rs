use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum VerificationError {
        HttpRequest(err: surf::Error) {
            display("{}", err)
        }
        Integrity(err: jwt_compact::ValidationError) {
            display("{}", err)
        }
        InvalidIssuerUrl {
            display("issuer URL mismatch")
        }
        InvalidIssueTime {
            display("invalid issue time")
        }
        MissingGroups {
            display("missing groups")
        }
        InvalidGroups {
            display("invalid groups")
        }
        UnsupportedAlgorithm {
            display("only RS256, RS384, and RS512 are supported")
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
    }
}
