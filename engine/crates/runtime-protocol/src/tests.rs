use std::borrow::Cow;

use engine::registry::{Registry, VersionedRegistry};
use sha2::{Digest, Sha256};

use crate::customer_deployment_config::{CommonCustomerDeploymentConfig, CustomerDeploymentConfig};

const EXPECTED_SHA: &str = "e59411b4671634d0e30c49df8817b98daa597fff6de4c2da77b80ccc21876b3c";

#[test]
fn test_serde_roundtrip() {
    let id = r#"
            This test ensures the default `VersionedRegistry` serialization output remains stable.

            When this test fails, it likely means the shape of the `Registry` type was updated,
            which can cause backward-incompatibility issues.

            Before updating this test to match the expected result, please ensure the changes to
            `Registry` are applied in a backward compatible way.

            One way to do so, is to have the `Default` trait return a value that keeps the existing
            expectation, and `#[serde(default)]` is applied to any newly added field.

            Once you are satisfied your changes are backward-compatible, update `EXPECTED_SHA` with
            the new output presented in the test result.
        "#;

    let registry = Cow::Owned(Registry::new().with_sample_data());
    let versioned_registry = VersionedRegistry {
        registry,
        deployment_id: Cow::Borrowed(id),
    };
    let serialized_versioned_registry = serde_json::to_string(&versioned_registry).unwrap();
    let serialized_sha = Sha256::digest(serialized_versioned_registry);

    assert_eq!(&format!("{serialized_sha:x}"), EXPECTED_SHA);
}

#[test]
fn serialize_customer_deployment_config() {
    use std::collections::HashMap;

    use common_types::UdfKind;
    let customer_gateway_config: CustomerDeploymentConfig<crate::LocalSpecificConfig> = CustomerDeploymentConfig {
        common: CommonCustomerDeploymentConfig {
            udf_bindings: HashMap::from([((UdfKind::Authorizer, "name".to_string()), "value".to_string())]),
            ..Default::default()
        },
        ..Default::default()
    };
    serde_json::to_string(&customer_gateway_config).unwrap();
}
