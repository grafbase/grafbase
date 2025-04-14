# Grafbase SDK for Gateway Extensions

[![docs.rs](https://img.shields.io/docsrs/grafbase-sdk)](https://docs.rs/grafbase-sdk)

This crate provides building blocks for creating [Grafbase Gateway](https://grafbase.com/docs/reference/gateway/installation) extensions.

There exist four kinds of extensions today:

- [AuthenticationExtension]: Authenticates clients before any GraphQL processing, generating a token with custom data for further extensions.
- [AuthorizationExtension]: Control access to certain fields, objects, interfaces, scalars or enums.
- [FieldResolverExtension]: Called by the gateway to resolve data for a field, replacing or augmenting a GraphQL subgraph.
- [SelectionSetResolverExtension]: Called by the gateway to resolve data downstream from a field, replacing or augmenting a GraphQL subgraph. These extensions have access to the selection set on the field and to the whole GraphQL schema.

Each extension has its dedicated documentation and tutorial you can follow through.
