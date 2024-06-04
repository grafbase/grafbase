#![allow(unused_crate_dependencies)]

const SCHEMA: &str = r#"
    type Query {
        user: User @resolver(name: "whatever")
    }

    type User {
        name: String @cache(maxAge: 140)
        email(domain: String): String @cache(maxAge: 130)
        someConstant: String @cache(maxAge: 120)
        nested: String @resolver(name: "whatever")
    }

    type Nested {
        someThing: String @cache(maxAge: 140)
        uncached(arg: String): String
    }
"#;

#[test]
fn test_basic_split() {
    let registry = build_registry(SCHEMA);

    const QUERY: &str = "query { user { name email someConstant } }";

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    let result = plan
        .cache_partitions
        .into_iter()
        .map(|(_, query)| query.as_display(&plan.document).to_string())
        .collect::<Vec<_>>();

    assert_eq!(result.len(), 3, "{result:?}");

    insta::assert_snapshot!(result[0], @r###"
    query {
      user {
        name
      }
    }
    "###);

    insta::assert_snapshot!(result[1], @r###"
    query {
      user {
        email
      }
    }
    "###);

    insta::assert_snapshot!(result[2], @r###"
    query {
      user {
        someConstant
      }
    }
    "###);
}

#[test]
fn test_split_with_absurd_fragments() {
    let registry = build_registry(SCHEMA);

    const QUERY: &str = r#"
        query {
            ...UserFragment
            user {
                name
                email
                ...ConstantFragment
            }
        }

        fragment UserFragment on Query {
            user {
                name
                ...NameAndEmailFragment
                ...EmailFragment
                ...ConstantFragment
            }
        }

        fragment NameAndEmailFragment on User {
            name
            ...EmailFragment
        }

        fragment EmailFragment on User {
            email
        }

        fragment ConstantFragment on User {
            someConstant
        }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    let result = plan
        .cache_partitions
        .into_iter()
        .map(|(_, query)| query.as_display(&plan.document).to_string())
        .collect::<Vec<_>>();

    assert_eq!(result.len(), 3, "{result:?}");

    // This one should have all the references to user
    insta::assert_snapshot!(result[0], @r###"
    query {
      ... UserFragment
      user {
        name
      }
    }

    fragment UserFragment on Query {
      user {
        name
        ... NameAndEmailFragment
      }
    }

    fragment NameAndEmailFragment on User {
      name
    }
    "###);

    // This one should have all the references to email
    insta::assert_snapshot!(result[1], @r###"
    query {
      ... UserFragment
      user {
        email
      }
    }

    fragment EmailFragment on User {
      email
    }

    fragment NameAndEmailFragment on User {
      ... EmailFragment
    }

    fragment UserFragment on Query {
      user {
        ... NameAndEmailFragment
        ... EmailFragment
      }
    }
    "###);

    // This one should have all the references to someConstant
    insta::assert_snapshot!(result[2], @r###"
    query {
      ... UserFragment
      user {
        ... ConstantFragment
      }
    }

    fragment ConstantFragment on User {
      someConstant
    }

    fragment UserFragment on Query {
      user {
        ... ConstantFragment
      }
    }
    "###);
}

#[test]
fn test_arguments_and_directives_preserved() {
    let registry = build_registry(
        r#"
        type Query {
            user: String @resolver(name: "whatever")
        }
        type Mutation {
            createUser: String @resolver(name: "whatever")
        }
        "#,
    );

    const QUERY: &str = r#"
    query @whatever {
        user(id: 1) @whatever {
            ... @defer(if: $hello) {
                hello
            }
            ...Whatever @whatever
            ... on User @defer(if: $hello) {
                hello
            }
        }
    }

    fragment Whatever on User @whatever {
        hello
    }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    assert!(plan.cache_partitions.is_empty());

    let result = plan.nocache_partition.as_display(&plan.document).to_string();

    insta::assert_snapshot!(result)
}

#[test]
fn test_mutation() {
    let registry = build_registry(
        r#"
        type Query {
            user: String @resolver(name: "whatever")
        }
        type Mutation {
            createUser: String @resolver(name: "whatever")
        }
        "#,
    );

    const MUTATION: &str = "mutation { createUser }";

    assert!(partial_caching::build_plan(MUTATION, None, &registry)
        .unwrap()
        .is_none());
}

#[test]
fn test_subscription() {
    let registry = build_registry(
        r#"
        type Query {
            user: String @resolver(name: "whatever")
        }
        type Mutation {
            createUser: String @resolver(name: "whatever")
        }
        "#,
    );

    const SUBSCRIPTION: &str = "subscription { createUser }";

    assert!(partial_caching::build_plan(SUBSCRIPTION, None, &registry)
        .unwrap()
        .is_none());
}

#[test]
fn test_split_with_variables() {
    let registry = build_registry(SCHEMA);

    const QUERY: &str = r#"
        query($theVar: String) {
            user {
                email(domain: $theVar)
                nested { uncached(arg: $theVar) }
            }
        }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    assert_eq!(plan.cache_partitions.len(), 1);

    insta::assert_snapshot!(plan.cache_partitions[0].1.as_display(&plan.document), @r###"
    query($theVar: String) {
      user {
        email(domain: $theVar)
      }
    }
    "###);

    insta::assert_snapshot!(plan.nocache_partition.as_display(&plan.document), @r###"
    query($theVar: String) {
      user {
        nested {
          uncached(arg: $theVar)
        }
      }
    }
    "###);
}

#[test]
fn test_cache_control_propagation() {
    let registry = build_registry(
        r#"
            type Query {
                user: User @resolver(name: "whatever")
                nested: Nested @resolver(name: "whatever")
            }

            type User @cache(maxAge: 60) {
                name: String
                nested: Nested @resolver(name: "whatever") @cache(maxAge: 80)
                other: Other!
            }

            type Nested @cache(maxAge: 120) {
                foo: String
                bar(arg: String): String
            }

            type Other {
                baz: String!
            }
            "#,
    );

    const QUERY: &str = r#"
        query {
            user {
                name
                nested {
                    foo
                    bar
                }
                other {
                    bax
                }
            }
            nested {
                foo
            }
        }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    assert_eq!(plan.cache_partitions.len(), 3);
    assert!(plan.nocache_partition.is_empty());

    assert_eq!(plan.cache_partitions[0].0.max_age, 60);
    insta::assert_snapshot!(plan.cache_partitions[0].1.as_display(&plan.document), @r###"
    query {
      user {
        name
        other {
          bax
        }
      }
    }
    "###);

    assert_eq!(plan.cache_partitions[1].0.max_age, 80);
    insta::assert_snapshot!(plan.cache_partitions[1].1.as_display(&plan.document), @r###"
    query {
      user {
        nested {
          foo
          bar
        }
      }
    }
    "###);

    assert_eq!(plan.cache_partitions[2].0.max_age, 120);
    insta::assert_snapshot!(plan.cache_partitions[2].1.as_display(&plan.document), @r###"
    query {
      nested {
        foo
      }
    }
    "###);
}

#[test]
fn test_cache_control_propagation_from_root() {
    let registry = build_registry(
        r#"
            extend schema @cache(
                rules: [
                    {maxAge: 60, types: [{name: "Query"}]},
                ]
            )

            type Query {
                user: User @resolver(name: "whatever")
            }
            type User {
                name: String
            }
    "#,
    );

    const QUERY: &str = "query { user { name } }";

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    assert!(plan.nocache_partition.is_empty());
    assert_eq!(plan.cache_partitions.len(), 1);

    assert_eq!(plan.cache_partitions[0].0.max_age, 60);
    insta::assert_snapshot!(plan.cache_partitions[0].1.as_display(&plan.document), @r###"
    query {
      user {
        name
      }
    }
    "###);
}

#[test]
fn test_cache_control_propagation_across_fragments() {
    let registry = build_registry(
        r#"
            type Query {
                user: User @resolver(name: "whatever")
                nested: Nested @resolver(name: "whatever")
            }

            type User @cache(maxAge: 60) {
                name: String
                nested: Nested @resolver(name: "whatever") @cache(maxAge: 80)
            }

            type Nested @cache(maxAge: 120) {
                foo: String
                bar(arg: String): String
            }
            "#,
    );

    const QUERY: &str = r#"
        query {
            user {
                ...UserFragment
            }
            nested {
                ...NestedFragment
            }
        }

        fragment UserFragment on User {
            name
            nested {
                ...NestedFragment
            }
        }

        fragment NestedFragment on Nested {
            foo
        }
    "#;

    let plan = partial_caching::build_plan(QUERY, None, &registry).unwrap().unwrap();

    assert_eq!(plan.cache_partitions.len(), 3);
    assert!(plan.nocache_partition.is_empty());

    assert_eq!(plan.cache_partitions[0].0.max_age, 120);
    insta::assert_snapshot!(plan.cache_partitions[0].1.as_display(&plan.document), @r###"
    query {
      nested {
        ... NestedFragment
      }
    }

    fragment NestedFragment on Nested {
      foo
    }
    "###);

    assert_eq!(plan.cache_partitions[1].0.max_age, 60);
    insta::assert_snapshot!(plan.cache_partitions[1].1.as_display(&plan.document), @r###"
    query {
      user {
        ... UserFragment
      }
    }

    fragment UserFragment on User {
      name
    }
    "###);

    assert_eq!(plan.cache_partitions[2].0.max_age, 80);
    insta::assert_snapshot!(plan.cache_partitions[2].1.as_display(&plan.document), @r###"
    query {
      user {
        ... UserFragment
      }
    }

    fragment NestedFragment on Nested {
      foo
    }

    fragment UserFragment on User {
      nested {
        ... NestedFragment
      }
    }
    "###);
}

fn build_registry(schema: &str) -> registry_for_cache::PartialCacheRegistry {
    registry_upgrade::convert_v1_to_partial_cache_registry(parser_sdl::parse_registry(schema).unwrap()).unwrap()
}
