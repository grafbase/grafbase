use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Duration, Utc};
use ring::{
    rand::SecureRandom,
    signature::{
        EcdsaKeyPair, EcdsaSigningAlgorithm, EcdsaVerificationAlgorithm, UnparsedPublicKey, ECDSA_P256_SHA256_ASN1,
        ECDSA_P256_SHA256_ASN1_SIGNING,
    },
};

/// The signing algorithm
pub static SIGNING_ALGORITHM: &EcdsaSigningAlgorithm = &ECDSA_P256_SHA256_ASN1_SIGNING;

/// The verification algorithm
pub static VERIFICATION_ALGORITHM: &EcdsaVerificationAlgorithm = &ECDSA_P256_SHA256_ASN1;

const GRACE_PERIOD_DAYS: i64 = 30;
const LICENSE_VALID_WEEKS: i64 = 52;

/// A Grafbase license for self-hosted gateway
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct License {
    account_slug: String,
    graph_slug: String,
    expire_timestamp: DateTime<Utc>,
}

impl License {
    /// Creates a new license
    pub fn new(account_slug: String, graph_slug: String, contract_start: DateTime<Utc>) -> Self {
        let expire_timestamp = contract_start + Duration::try_weeks(LICENSE_VALID_WEEKS).expect("must work");

        Self {
            account_slug,
            graph_slug,
            expire_timestamp,
        }
    }

    /// The account of the license
    pub fn account_slug(&self) -> &str {
        &self.account_slug
    }

    /// The graph of the license
    pub fn graph_slug(&self) -> &str {
        &self.graph_slug
    }

    /// True, if the license is in a 30-day grace period.
    pub fn in_grace_period(&self) -> bool {
        (self.expire_timestamp - Utc::now()).num_days() < GRACE_PERIOD_DAYS
    }

    /// True, if the license is expired.
    pub fn expired(&self) -> bool {
        (self.expire_timestamp - Utc::now()).num_minutes() < 0
    }

    /// Creates a signed license, which should be sent as a response for a license query.
    pub fn sign(&self, private_key: &[u8], rng: &dyn SecureRandom) -> crate::Result<SignedLicense> {
        let key_pair = EcdsaKeyPair::from_pkcs8(SIGNING_ALGORITHM, private_key, rng)
            .map_err(|_| crate::Error::InvalidSigningKey)?;

        let license = serde_json::to_string(&self).expect("must be a valid license");
        let license = general_purpose::STANDARD.encode(license.into_bytes());

        let signature = key_pair
            .sign(rng, license.as_bytes())
            .map_err(|_| crate::Error::SigningFailed)?;

        let signature = general_purpose::STANDARD.encode(signature.as_ref());

        Ok(SignedLicense { license, signature })
    }
}

/// A signed license. This should be stored in the license file.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignedLicense {
    license: String,
    signature: String,
}

impl SignedLicense {
    /// Verifies the license with a public key. If the signature validates, returns a license structure.
    pub fn verify(&self, public_key: &[u8]) -> crate::Result<License> {
        let signature = general_purpose::STANDARD
            .decode(&self.signature)
            .map_err(|_| crate::Error::InvalidLicense)?;

        UnparsedPublicKey::new(VERIFICATION_ALGORITHM, public_key)
            .verify(self.license.as_bytes(), &signature)
            .map_err(|_| crate::Error::InvalidLicense)?;

        let license = general_purpose::STANDARD
            .decode(&self.license)
            .map_err(|_| crate::Error::InvalidLicense)?;

        let license: License = serde_json::from_slice(&license).map_err(|_| crate::Error::InvalidLicense)?;

        if license.expired() {
            return Err(crate::Error::InvalidLicense);
        }

        Ok(license)
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose, Engine};
    use chrono::{Duration, Utc};
    use ring::rand::SystemRandom;

    use crate::{keys, License};

    struct TestKey {
        private_key: &'static [u8],
        public_key: &'static [u8],
    }

    fn load_test_key() -> TestKey {
        let private_key = keys::private_key();
        let public_key = keys::public_key();

        TestKey {
            private_key,
            public_key,
        }
    }

    #[test]
    fn valid_license() {
        let key = load_test_key();
        let contract_start = Utc::now();

        let rng = SystemRandom::new();
        let license = License::new(String::from("account"), String::from("graph"), contract_start);
        let signed = license.sign(key.private_key, &rng).unwrap();
        let verified = signed.verify(key.public_key).unwrap();

        assert!(!verified.expired());
        assert!(!verified.in_grace_period());

        assert_eq!(license, verified);
    }

    #[test]
    fn grace_period() {
        let key = load_test_key();
        let contract_start = Utc::now() - Duration::try_weeks(51).unwrap();

        let rng = SystemRandom::new();
        let license = License::new(String::from("account"), String::from("graph"), contract_start);
        let signed = license.sign(key.private_key, &rng).unwrap();
        let verified = signed.verify(key.public_key).unwrap();

        assert!(!verified.expired());
        assert!(verified.in_grace_period());

        assert_eq!(license, verified);
    }

    #[test]
    fn expired() {
        let key = load_test_key();
        let contract_start = Utc::now() - Duration::try_weeks(53).unwrap();

        let rng = SystemRandom::new();
        let license = License::new(String::from("account"), String::from("graph"), contract_start);
        let signed = license.sign(key.private_key, &rng).unwrap();
        let error = signed.verify(key.public_key).unwrap_err();

        assert_eq!(crate::Error::InvalidLicense, error);
    }

    #[test]
    fn tampered() {
        let key = load_test_key();
        let contract_start = Utc::now();

        let tampered = License::new(String::from("meow"), String::from("purr"), contract_start);
        let tampered = serde_json::to_string(&tampered).expect("must be a valid license");
        let tampered = general_purpose::STANDARD.encode(tampered.into_bytes());

        let license = License::new(String::from("account"), String::from("graph"), contract_start);

        let rng = SystemRandom::new();
        let mut signed = license.sign(key.private_key, &rng).unwrap();
        signed.license = tampered;

        let error = signed.verify(key.public_key).unwrap_err();

        assert_eq!(crate::Error::InvalidLicense, error);
    }
}
