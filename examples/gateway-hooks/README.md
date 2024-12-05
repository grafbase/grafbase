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
- Cargo component on Rust 1.82 or older ([install docs](https://github.com/bytecodealliance/cargo-component?tab=readme-ov-file#installation))
- Rust target wasm32-wasip2 for Rust 1.83 or later (`rustup target add wasm32-wasip2`)
- A GraphQL client, such as [Altair](https://altair-gql.sirmuel.design/)

For the advanced users using nix with flakes support:

```
nix develop
```

### Running the example

First, start the subgraph in one terminal:

```bash
docker compose up --force-recreate --build -d
```

Next compile the WebAssembly hook functions into a Wasm component in another terminal.

On Rust 1.83 or later:

```bash
cd hooks
cargo build --target wasm32-wasip2
```

On Rust 1.82 or earlier:

```bash
cd hooks
cargo component build --release
```

After a successful build, the Wasm component should be located at `target/wasm32-wasip(1 or 2)/release/demo_hooks.wasm`.

The `grafbase-gateway` is already started in the docker compose file, restart it to take the hook changes into account:

```bash
docker compose restart gateway
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
    --data '{"query": "query { user(id: 2) { name address { street } } }"}' \
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
