use std::collections::BTreeMap;

use super::*;

#[test]
fn test_stripe_federation_schema() {
    super::init_tracing();

    let metadata = ApiMetadata {
        url: None,
        ..metadata("stripe", true)
    };

    let registry = build_registry("test_data/stripe.openapi.json", Format::Json, metadata).unwrap();

    insta::assert_json_snapshot!(registry.federation_entities.into_iter().collect::<BTreeMap<_, _>>());
}
