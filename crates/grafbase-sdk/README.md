# Grafbase Gateway SDK for Extensions

[![docs.rs](https://img.shields.io/docsrs/grafbase-sdk)](https://docs.rs/grafbase-sdk)

This crate provides building blocks for creating [Grafbase Gateway](https://grafbase.com/docs/reference/gateway/installation) extensions.

## Usage

**Extensions are still under development. Expect issues if you try them out before we complete development.**

Initialize a new project with the [Grafbase CLI](https://grafbase.com/docs/reference/grafbase-cli):

```bash,no_run
grafbase extension init --type auth/resolver my-extension
```

This creates a new project with the necessary files and dependencies to get you started. Edit the `src/lib.rs` file to add your extension logic. The Grafbase Gateway initializes the struct `TestProject` once during the initial extension call. The Gateway maintains extensions in a connection pool and reuses the struct for multiple requests. Because an extension runs single-threaded, we maintain multiple instances in the gateway memory to handle multiple requests concurrently.

### Resolver Example

You can initialize a new resolver extension with the Grafbase CLI:

```bash
grafbase extension init --type resolver my-extension
```

The initialization accepts a list of schema directives from the federated schema (defined in the schema file) and a [`Configuration`](types::Configuration) object that remains empty for resolver extensions. The [`ResolverExtension`] derive macro generates the necessary code to initialize a resolver extension and guides you to implement two traits: [`Extension`] and [`Resolver`]. The [`Extension`] trait initializes the extension, and the [`Resolver`] trait implements the extension logic to resolve a field:

```rust
# use grafbase_sdk::{
#    types::{Configuration, Directive, FieldDefinition, FieldInputs, FieldOutput},
#    Error, Extension, Resolver, ResolverExtension, SharedContext,
# };
#[derive(ResolverExtension)]
struct TestProject;

impl Extension for TestProject {
    fn new(schema_directives: Vec<Directive>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }
}

impl Resolver for TestProject {
    fn resolve_field(
        &mut self,
        context: SharedContext,
        directive: Directive,
        field_definition: FieldDefinition,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error> {
        todo!()
    }
}
```

The `schema_directives` in the constructor provides serialized access to all the `SCHEMA` directives from the subgraph SDL defined in the `definitions.graphql` file. The `directive` in the `resolve_field` provides serialized access to the directive that triggered the resolver extension.

The [`FieldOutput`](types::FieldOutput) contains the serialized output of the resolver which transfers back to the gateway. Remember to match the serialized response to the type of the field resolver.

You can find a full [example](https://github.com/grafbase/grafbase/blob/main/extensions/rest/) of a REST resolver extension in the Grafbase repository.

### Authentication Example

You can initialize a new authentication extension with the Grafbase CLI:

```bash
grafbase extension init --type auth my-extension
```

The initialization needs a list of schema directives from the federated schema (empty for authentication extensions) and a [`Configuration`](types::Configuration) object that reflects the extension provider configuration in `grafbase.toml`. The [`AuthenticationExtension`] derive macro generates code to initialize a resolver extension and guides you to implement two traits: [`Extension`] and [`Authenticator`]. The [`Extension`] trait initializes the extension, and the [`Authenticator`] trait implements the extension logic to authenticate a request:

```rust
use grafbase_sdk::{
    types::{Configuration, Directive, ErrorResponse, Token},
    AuthenticationExtension, Authenticator, Extension, Headers,
};

#[derive(AuthenticationExtension)]
struct TestProject;

impl Extension for TestProject {
    fn new(schema_directives: Vec<Directive>, config: Configuration) -> Result<Self, Box<dyn std::error::Error>>
        where
            Self: Sized,
        {
            todo!()
        }
    }

impl Authenticator for TestProject {
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
        todo!()
    }
}
```

The system deserializes the configuration from the `grafbase.toml` configuration. As an example, here's the configuration data from the JWT extension:

```toml
[[authentication.providers]]

[authentication.providers.extension]
extension = "jwt"

[authentication.providers.extension.config]
url = "https://example.com/.well-known/jwks.json"
issuer = "example.com"
audience = "my-project"
poll_interval = 60
header_name = "Authorization"
header_value_prefix = "Bearer "
```

The `config` section becomes available through the [`Configuration`](types::Configuration) struct, and structs implementing [`serde::Deserialize`](https://docs.rs/serde/latest/serde/derive.Deserialize.html) can deserialize it using with the correspondig [deserialization method](types::Configuration::deserialize).

The `authenticate` method receives request headers as input. A returned token allows the request to continue. You can serialize any data with [`serde::Serialize`](https://docs.rs/serde/latest/serde/derive.Serialize.html) and pass it to the [token initializer](types::Token::new). For certain directives like [`@requiredScopes`](https://grafbase.com/docs/reference/graphql-directives#requiresscopes), define the `scope` claim in the token.

Find a complete [example](https://github.com/grafbase/grafbase/blob/main/extensions/jwt/) of a JWT authentication extension in the Grafbase repository.

## Building

You can build your extension with the Grafbase CLI. For this to work, you must have a working [rustup](https://rustup.rs/) installation:

```bash,ignore
grafbase extension build
```

This compiles your extension and creates two files:

```text
build/
├── manifest.json
└── test_project.wasm
```

You can use the path to the `build` directory in your gateway configuration to try out the extension.

## Testing

You can enable the `test-utils` feature for this crate in your extension. The feature provides tooling for testing the extensions against the Grafbase Gateway. Keep in mind to add it as a `dev` dependency, the test utils do _not_ compile to WebAssembly. Your tests run as native code, and only the extension compiles into a WebAssembly component and tests with the gateway binary.

```bash
cargo add --dev grafbase-gateway --features test-utils
```

Write your tests in the `tests/integration_tests.rs` file in your extension project.

See the [integration tests](https://github.com/grafbase/grafbase/blob/main/extensions/rest/tests/integration_tests.rs) of the REST extension for an example of how to use the test utils.

You can run the tests with `cargo`:

```bash
cargo test
```
