# Grafbase ⨯ Couchbase

This example shows how to create a Grafbase resolver that can execute read and write operations to Couchbase &mdash; [read guide](https://grafbase.com/guides/build-graphql-apis-at-the-edge-with-couchbase).

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-couchbase grafbase-with-resolvers-couchbase` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-couchbase`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [Couchbase](https://www.couchbase.com) Query Service URL, Username and Password
5. Run `npx grafbase dev` to start local dev server with your schema
6. Open [Pathfinder](http://localhost:4000)
