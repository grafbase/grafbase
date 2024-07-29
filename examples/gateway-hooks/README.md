# Gateway hooks example

In this example we will use the gateway hooks to change the behavior of the gateway. This repository contains the following:

- `subgraph`: a GraphQL server exposing a few users
- `auth-service`: a HTTP service imitating an authorization endpoint to grant access to some data.
- `federated-schema.graphql`: the federated GraphQL schema generated with the `subgraph`.
- `grafbase.toml`: the configuration for the Grafbase Gateway`

## Setup

### Dependencies

To run this example, you need the Grafbase Gateway version 0.4.0 or later, read more how to install it from:

https://grafbase.com/docs/self-hosted-gateway

Additionally, the following tools are needed:

- A C compiler, such as clang together with pkg-config (install based on your system, `cc` command is required)
- If on Linux, cargo-component depends on OpenSSL (`libssl-dev` on Debian)
- Rust compiler ([install docs](https://www.rust-lang.org/learn/get-started))
- Cargo component ([install docs](https://github.com/bytecodealliance/cargo-component?tab=readme-ov-file#installation))
- A GraphQL client, such as [Altair](https://altair-gql.sirmuel.design/)

For the advanced users using nix with flakes support:

```
nix develop
```

### Running the example

First, start the subgraph in one terminal:

```bash
cd subgraph
cargo run --release
```

Then the authorization service:

```sh
cd auth-service
cargo run --release
```

Next compile the WebAssembly hook functions into a Wasm component in another terminal:

```bash
cd hooks
cargo component build --release
```

After a successful build, the Wasm component should be located at `target/wasm32-wasip1/release/demo_hooks.wasm`.

Finally start the `grafbase-gateway`:

```bash
grafbase-gateway --schema federated-schema.graphql --config grafbase.toml
```

Now you are ready to send queries!

## Design

The hooks implement the following authorization rules:

1. An user with id N can see all users with an ID equal or inferior to N: User 3 can see users 1, 2 and 3 but not 4
2. An admin can see the list of all users (header `x-role: admin`)
3. The address is only available to the user himself

The header `x-current-user-id` determines the current user id and `x-role` defines the role.

### Examples

Can not access any user data:

```sh
curl -X POST http://127.0.0.1:5000/graphql \
    --data '{"query": "query { user(id: 1) { name } }"}' \
    -H 'Content-Type: application/json'
```

Can access one's own data:

```sh
curl -X POST http://127.0.0.1:5000/graphql \
    --data '{"query": "query { user(id: 3) { name address { street } } }"}' \
    -H 'Content-Type: application/json' \
    -H 'x-current-user-id: 2'
```

Can access user name from 1 & 2, but not 3 & 4, and only its own address:

```sh
curl -X POST http://127.0.0.1:5000/graphql \
    --data '{"query": "query { users { name address { street } } }"}' \
    -H 'Content-Type: application/json' \
    -H 'x-current-user-id: 2'
```

Can access all user names, but only its own address:

```sh
curl -X POST http://127.0.0.1:5000/graphql \
    --data '{"query": "query { users { name address { street } } }"}' \
    -H 'Content-Type: application/json' \
    -H 'x-current-user-id: 2' \
    -H 'x-role: admin'
```
