# Grafbase SDK for Gateway Extensions

[![docs.rs](https://img.shields.io/docsrs/grafbase-sdk)](https://docs.rs/grafbase-sdk)

This crate provides building blocks for creating [Grafbase Gateway](https://grafbase.com/docs/reference/gateway/installation) extensions.

There exist four kinds of extensions today:

- [AuthenticationExtension]: Authenticates clients before any GraphQL processing, generating a token with custom data for further extensions.
- [AuthorizationExtension]: Control access to certain fields, objects, interfaces, scalars or enums.
- [ResolverExtension]: Called by the gateway to resolve data for a field from a GraphQL subgraph.
- [HooksExtension]: Called by the gateway to perform actions before receiving a request, or right before sending a response.
- [ContractsExtension]: Called by the gateway to identify which elements should be part of a schema contract given subgraph directives and the contract key.

Each extension has its dedicated documentation and tutorial you can follow through.
