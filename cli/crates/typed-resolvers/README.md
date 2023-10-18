# typed-resolvers

This crate implements code generation and resolver discovery for custom resolvers.

## Design

This crate does not reuse existing infrastructure because the amount of
analysis required on the schema is very limited: we do not want to do extra
validation because we want to tolerate fairly broken schemas and still generate
code. Also code generation has to be fast for integration in the `gb dev` loop.

Unlike graphql-code-generator, we do not generate types for the resolver
functions. By design, the generated module has one TypeScript type per GraphQL
type. They have the same name (except TS reserved keywords), and they are in
the same order in the generated file as in the source file.

## Tests

### Overview

The tests are located in the `tests/` directory.

Each test is a GraphQL file. When you run `cargo test`, the typescript module
is generated for each graphql file and compared with the matching
`.expected.ts` snapshot. Test discovery is implemented in `build.rs` and the
test runner in `tests/`.

### Updating the snapshots

Run `cargo test` with the `UPDATE_EXPECT` env var defined.
