# Authenticated

Provides an `@authenticated` directive which prevents access to elements in the query when the user is not authenticated.

## Install

```toml
[extension.authenticated]
version = "1.0"
```

## Usage

```graphql
extend schema @link(url: "https://grafbase.com/extensions/authenticated/1.0.0", import: ["@authenticated"])

type Query {
  public: String!
  mustBeAuthenticated: String! @authenticated
}
```
