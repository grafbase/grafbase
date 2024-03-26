use chrono::{DateTime, Utc};
use jwt_simple::{
    algorithms::{ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair, ES256PublicKey},
    claims::{Claims, JWTClaims},
    common::VerificationOptions,
};
use ulid::Ulid;

static ISSUER: &str = "Grafbase";

const GRACE_PERIOD_DAYS: u64 = 30;
const LICENSE_VALID_DAYS: i64 = 365;

/// A Grafbase license for self-hosted gateway
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct License {
    /// The ID of the graph this license is generated for
    pub graph_id: Ulid,
    /// The ID of the account this license is generated for
    pub account_id: Ulid,
}

impl License {
    /// Creates a signed license, which should be sent as a response for a license query.
    pub fn sign(self, key_pair: &ES256KeyPair, contract_signing_date: DateTime<Utc>) -> crate::Result<String> {
        let issued_at = Utc::now();

        let valid_for =
            contract_signing_date - issued_at + chrono::Duration::try_days(LICENSE_VALID_DAYS).expect("must work");

        if valid_for.num_milliseconds().is_negative() {
            return Err(crate::Error::SigningFailed);
        }

        let valid_for = u64::try_from(valid_for.num_milliseconds()).expect("has to be positive");
        let valid_for = jwt_simple::prelude::Duration::from_millis(valid_for);
        let claims = Claims::with_custom_claims(self, valid_for).with_issuer(ISSUER);

        key_pair.sign(claims).map_err(|_| crate::Error::SigningFailed)
    }

    /// Verify a licens with the public key
    pub fn verify(token: &str, public_key: &ES256PublicKey) -> crate::Result<JWTClaims<License>> {
        let options = VerificationOptions {
            time_tolerance: Some(jwt_simple::prelude::Duration::from_days(GRACE_PERIOD_DAYS)),
            ..Default::default()
        };

        public_key
            .verify_token(token, Some(options))
            .map_err(|_| crate::Error::InvalidLicense)
    }
}

/// True, if the given claim has an expiry date and it's in the grace period.
pub fn in_grace_period(claims: &JWTClaims<License>) -> bool {
    match claims.expires_at {
        Some(expiry) => {
            let days_left = expiry.as_secs() - u64::try_from(Utc::now().timestamp()).expect("must work");
            days_left < (GRACE_PERIOD_DAYS * 24 * 60 * 60)
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, process::Command};

    use chrono::{Duration, Utc};
    use jwt_simple::algorithms::{ES256KeyPair, ES256PublicKey};
    use ulid::Ulid;

    use crate::{license::in_grace_period, License};

    struct TestKey {
        private_key: ES256KeyPair,
        public_key: ES256PublicKey,
    }

    fn load_test_key() -> TestKey {
        let output = Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;

        let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
        let workspace_dir = cargo_path.parent().unwrap().to_path_buf();
        let private_key_path = workspace_dir.join("engine/crates/licensing/test/private-test-key.pem");

        let private_key = fs::read_to_string(private_key_path).unwrap();
        let private_key = ES256KeyPair::from_pem(&private_key).unwrap();
        let public_key = private_key.public_key();

        TestKey {
            private_key,
            public_key,
        }
    }

    #[test]
    fn valid_license() {
        let key = load_test_key();
        let contract_start = Utc::now();

        let license = License {
            graph_id: Ulid::new(),
            account_id: Ulid::new(),
        };

        let token = license.clone().sign(&key.private_key, contract_start).unwrap();
        let verified = License::verify(&token, &key.public_key).unwrap();

        assert!(!in_grace_period(&verified));

        assert_eq!(license, verified.custom);
    }

    #[test]
    fn grace_period() {
        let key = load_test_key();
        let contract_start = Utc::now() - Duration::try_days(365).unwrap();

        let license = License {
            graph_id: Ulid::new(),
            account_id: Ulid::new(),
        };

        let token = license.clone().sign(&key.private_key, contract_start).unwrap();
        let verified = License::verify(&token, &key.public_key).unwrap();

        assert!(in_grace_period(&verified));

        assert_eq!(license, verified.custom);
    }

    #[test]
    fn tampered() {
        let key = load_test_key();
        let error = License::verify("false", &key.public_key).unwrap_err();

        assert_eq!(crate::Error::InvalidLicense, error);
    }
}
