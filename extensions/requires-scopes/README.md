# RequiresScopes

Provides an `@requiresScopes` directive which prevents access to elements in the query if the user doesn't have the right OAuth scopes. It expects the authentication token to be in JSON, as provided by the [jwt](https://grafbase.com/extensions/jwt) extension, and have the scopes in OAuth2 format. So a `scope` claim with a list of scopes as a string separated by a space.

## Install

```toml
[extension.requires-scopes]
version = "1.0"
```

## Usage

```graphql
extend schema
  @link(url: "https://grafbase.com/extensions/requires-scopes/1.0.0", import: ["@requiresScopes"])

type Query {
  public: String!
  hasReadScope: String @requiresScopes(scopes: "read")
  hasReadAndWriteScope: String @requiresScopes(scopes: [["read", "write"]])
  hasReadOrWriteScope: String @requiresScopes(scopes: [["read"], ["write"]])
}
```
