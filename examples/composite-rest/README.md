# Composite REST Example

This example demonstrates how to build a GraphQL service exposing REST APIs without any subgraphs with the help of our extensions and the composite schemas specification. There are three APIs:

- countries: Simple endpoint using restcountries.com and our REST extension.
- geo-api: Uses an public open data endpoint with a custom extension that relies on its convention to show case how this could enable you to create a simpler REST extension for your endpoints.
- zendesk: Uses the Zendesk Sales CRM API, a mock of it to be more precise, with the REST extension to showcase all of the different possibilities of joining your data with the help of a composite schema specification. With and without batching.


The structure of the repository is as follows:

- `subgraphss/*`: the different subgraphs being federated by the gateway.
- `extension/*`: the geo extension

The `test.hurl` files contains a list of HTTP requests and responses presenting all the functionality.
