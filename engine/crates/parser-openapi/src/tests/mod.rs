use std::sync::Once;

use assert_matches::assert_matches;
use engine::registry::{MetaType, UnionType};

use super::*;

mod federation;

#[test]
fn test_stripe_output() {
    let metadata = ApiMetadata {
        url: None,
        ..metadata("stripe", true)
    };
    insta::assert_snapshot!(build_registry("test_data/stripe.openapi.json", Format::Json, metadata)
        .unwrap()
        .export_sdl(false));
}

#[test]
fn test_stripe_output_json() {
    let metadata = ApiMetadata {
        url: None,
        ..metadata("stripe", true)
    };
    let registry = build_registry("test_data/stripe.openapi.json", Format::Json, metadata).unwrap();
    insta::assert_json_snapshot!(registry);
}

#[test]
fn test_petstore_output() {
    let registry = build_registry(
        "test_data/petstore.openapi.json",
        Format::Json,
        metadata("petstore", true),
    )
    .unwrap();

    insta::assert_snapshot!(registry.export_sdl(false));
    insta::assert_debug_snapshot!(registry);
}

#[test]
fn test_petstore_without_prefix() {
    let registry = build_registry(
        "test_data/petstore.openapi.json",
        Format::Json,
        ApiMetadata {
            type_prefix: None,
            ..metadata("petstore", false)
        },
    )
    .unwrap();

    insta::assert_snapshot!(registry.export_sdl(false));
}

#[test]
fn test_flat_output() {
    let registry = build_registry(
        "test_data/petstore.openapi.json",
        Format::Json,
        metadata("petstore", false),
    )
    .unwrap();

    insta::assert_snapshot!(registry.export_sdl(false));
    insta::assert_debug_snapshot!(registry);
}

#[test]
fn test_url_without_host_failure() {
    let metadata = ApiMetadata {
        url: None,
        ..metadata("petstore", true)
    };
    assert_matches!(
        build_registry("test_data/petstore.openapi.json", Format::Json, metadata)
            .unwrap_err()
            .as_slice(),
        [Error::InvalidUrl(url)] => {
            assert_eq!(url, "/api/v3");
        }
    );
}

#[test]
fn test_openai_output() {
    insta::assert_snapshot!(build_registry(
        "test_data/openai.yaml",
        Format::Yaml,
        ApiMetadata {
            query_naming: QueryNamingStrategy::OperationId,
            ..metadata("openai", true)
        }
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_planetscale() {
    insta::assert_snapshot!(build_registry(
        "test_data/planetscale.json",
        Format::Json,
        ApiMetadata {
            url: None,
            ..metadata("planetscale", true)
        }
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_orb() {
    // Orb is a 3.1 spec
    insta::assert_snapshot!(build_registry(
        "test_data/orb.json",
        Format::Json,
        ApiMetadata {
            url: None,
            ..metadata("orb", true)
        }
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_mongo_atlas() {
    init_tracing();

    // Mongo Atlas is a 3.1 spec
    insta::assert_snapshot!(build_registry(
        "test_data/mongo-atlas.json",
        Format::Json,
        ApiMetadata {
            url: None,
            ..metadata("mongo", true)
        }
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_impossible_unions() {
    insta::assert_snapshot!(build_registry(
        "test_data/impossible-unions.json",
        Format::Json,
        metadata("petstore", true)
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_all_of_schema_simple() {
    // an allOf schema that are simple merges of distinct objects
    insta::assert_snapshot!(build_registry(
        "test_data/all-ofs-simple.json",
        Format::Json,
        metadata("petstore", true)
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_all_of_schema_complex() {
    // Some allOf schemas have properties defined in multiple branches
    // of the allOf, with required sometiems being in one branch but not
    // another.  This is a test of that....
    insta::assert_snapshot!(build_registry(
        "test_data/all-ofs-complex.json",
        Format::Json,
        metadata("petstore", true)
    )
    .unwrap()
    .export_sdl(false));
}

#[test]
fn test_supabase() {
    insta::assert_snapshot!(
        build_registry("test_data/supabase.json", Format::Json, metadata("supabase", true))
            .unwrap()
            .export_sdl(false)
    );
}

#[test]
fn test_greenlake_schema() {
    init_tracing();

    let metadata = ApiMetadata {
        url: None,
        query_naming: QueryNamingStrategy::OperationId,
        ..metadata("greenlake", false)
    };

    let registry = build_registry("test_data/greenlake.yml", Format::Json, metadata).unwrap();

    insta::assert_snapshot!(registry.export_sdl(false));
}

#[test]
fn test_stripe_discrimnator_detection() {
    let registry = build_registry("test_data/stripe.openapi.json", Format::Json, metadata("stripe", true)).unwrap();
    let discriminators = registry
        .types
        .values()
        .filter_map(|ty| match ty {
            MetaType::Union(UnionType {
                name, discriminators, ..
            }) => Some((name, discriminators)),
            _ => None,
        })
        .collect::<Vec<_>>();

    insta::assert_json_snapshot!(discriminators);
}

fn build_registry(schema_path: &str, format: Format, metadata: ApiMetadata) -> Result<Registry, Vec<Error>> {
    let mut registry = Registry::new();

    parse_spec(
        std::fs::read_to_string(schema_path).unwrap(),
        format,
        metadata,
        &mut registry,
    )?;

    Ok(registry)
}

fn metadata(name: &str, namespace: bool) -> ApiMetadata {
    ApiMetadata {
        name: name.to_string(),
        namespace,
        url: Some(Url::parse("http://example.com").unwrap()),
        headers: ConnectorHeaders::new([]),
        query_naming: QueryNamingStrategy::SchemaName,
        type_prefix: Some(name.to_string()),
    }
}

fn init_tracing() {
    static INITIALIZER: Once = Once::new();
    INITIALIZER.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("trace")
            .without_time()
            .with_test_writer()
            .init();
    });
}
