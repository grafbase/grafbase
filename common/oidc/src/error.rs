use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum VerificationError {
        HttpRequest(err: surf::Error) {
            display("{}", err)
        }
        InvalidIssuer {
            display("issuer URL mismatch")
        }
        UnsupportedAlgorithm {
            display("only RS256 is supported right now")
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
