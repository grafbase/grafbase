use crate::gateway::extensions::field_resolver::validation::EchoExt;
use integration_tests::{gateway::Gateway, runtime};

#[test]
fn invalid_location_but_not_used() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query @echo {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION | OBJECT
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @"None");
    });
}

#[test]
fn invalid_location() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query @echo(input: "something") {
                    echo: JSON
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION | OBJECT
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query, for the extension 'echo-1.0.0' directive @echo: InputValueSet can only be used in directive applied on FIELD_DEFINITION, but found on OBJECT. See schema at 18:24:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {input: \"something\"})",
        )
        "#);
    });
}

#[test]
fn unknown_field() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "unknown")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet!) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Unknown input value 'unknown'. See schema at 19:92:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {input: \"unknown\"})",
        )
        "#);
    });
}

#[test]
fn unknown_nested_field() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "filters { nested { unknown } }")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Unknown input value 'unknown' at path '.filters.nested'. See schema at 19:92:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {input: \"filters { nested { unknown } }\"})",
        )
        "#);
    });
}

#[test]
fn cannot_have_selection_set() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "after { something }")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Type String cannot have a selecction set at path '.after'. See schema at 19:92:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {input: \"after { something }\"})",
        )
        "#);
    });
}

#[test]
fn cannot_have_fragments() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "filters { ... {  latest } }")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Cannot use fragments inside a InputValueSet. See schema at 19:92:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {input: \"filters { ... {  latest } }\"})",
        )
        "#);
    });
}

#[test]
fn must_be_valid_selection_set() {
    runtime().block_on(async move {
        let result = Gateway::builder()
            .with_subgraph_sdl(
                "a",
                r#"
                extend schema
                    @link(url: "echo-1.0.0", import: ["@echo"])

                scalar JSON

                type Query {
                    echo(first: Int, limit: Int, after: String, filters: Filters): JSON @echo(input: "filters {")
                }

                input Filters {
                    latest: Boolean
                    nested: Nested
                }

                input Nested {
                    id: ID
                    name: String
                }
                "#,
            )
            .with_extension(EchoExt::with_sdl(
                r#"
                extend schema @link(url: "https://specs.grafbase.com/grafbase", import: ["InputValueSet"])

                directive @echo(input: InputValueSet) on FIELD_DEFINITION
                "#,
            ))
            .try_build()
            .await;

        insta::assert_debug_snapshot!(result.err(), @r#"
        Some(
            "At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Could not parse InputValueSet: unexpected closing brace ('}') token (expected one of , \"...\"RawIdent, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, \"enum\", union, scalar, extend, directive, repeatable, on, fragment). See schema at 19:92:\n(graph: A, extension: ECHO, name: \"echo\", arguments: {input: \"filters {\"})",
        )
        "#);
    });
}
