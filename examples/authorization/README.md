# Custom authentication & authorization

[Tutorial](https://grafbase.com/blog/custom-authentication-and-authorization-in-graphql-federation)

This example presents how to setup custom authentication and authorization with extensions.
In this example authentication is fairly simple and will generate a custom token with the user id provided in the `current-user-id` header. If not present, authentication will fail. Authorization is more complex showcasing all the functionalities. The extension will:

- Update the subgraph headers to include the right `Authorization` header for subgraph depending on accessed data.
- Prevent access to users based on field arguments. The data will never be queried from the subgraph.
- Prevent access to accounts based on response data. In this case the gateway will provide response data to the extension to take a decision and will remove any denied elements from the response.

The structure of the repository is as follows:

- `auth-service`: A HTTP server acting as a authorization service
- `subgraphs/user`: the `user` subgraph being federated by the gateway.
- `extension/*`: the authentication & authorization extension

The `test.hurl` files contains a list of HTTP requests and responses presenting all the functionality.
