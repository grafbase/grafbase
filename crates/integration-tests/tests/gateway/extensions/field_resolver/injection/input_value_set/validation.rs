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

        if let Err(err) = result {
            panic!("{err}")
        }
    });
}

#[test]
fn invalid_location_but_not_used_nullable_with_default_value() {
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

                directive @echo(input: InputValueSet = "*") on FIELD_DEFINITION | OBJECT
                "#,
            ))
            .try_build()
            .await;

        if let Err(err) = result {
            panic!("{err}")
        }
    });
}

#[test]
fn invalid_location_but_not_used_required_with_default_value() {
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

                directive @echo(input: InputValueSet! = "*") on FIELD_DEFINITION | OBJECT
                "#,
            ))
            .try_build()
            .await;

        if let Err(err) = result {
            panic!("{err}")
        }
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query, for the extension 'echo-1.0.0' directive @echo: InputValueSet can only be used in directive applied on FIELD_DEFINITION, but found on OBJECT
        17 | type Query
        18 |   @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "something"})
                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        19 | {
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Unknown input value 'unknown'
        18 | {
        19 |   echo(first: Int, limit: Int, after: String, filters: Filters): JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "unknown"}) @join__field(graph: A)
                                                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 | }
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Unknown input value 'unknown' at path '.filters.nested'
        18 | {
        19 |   echo(first: Int, limit: Int, after: String, filters: Filters): JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "filters { nested { unknown } }"}) @join__field(graph: A)
                                                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 | }
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Type String cannot have a selecction set at path '.after'
        18 | {
        19 |   echo(first: Int, limit: Int, after: String, filters: Filters): JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "after { something }"}) @join__field(graph: A)
                                                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 | }
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Cannot use fragments inside a InputValueSet
        18 | {
        19 |   echo(first: Int, limit: Int, after: String, filters: Filters): JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "filters { ... {  latest } }"}) @join__field(graph: A)
                                                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 | }
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

        insta::assert_snapshot!(result.unwrap_err(), @r#"
        * At site Query.echo, for the extension 'echo-1.0.0' directive @echo: Could not parse InputValueSet: unexpected closing brace ('}') token (expected one of , "..."RawIdent, schema, query, mutation, subscription, ty, input, true, false, null, implements, interface, "enum", union, scalar, extend, directive, repeatable, on, fragment)
        18 | {
        19 |   echo(first: Int, limit: Int, after: String, filters: Filters): JSON @extension__directive(graph: A, extension: ECHO, name: "echo", arguments: {input: "filters {"}) @join__field(graph: A)
                                                                                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
        20 | }
        "#);
    });
}
