# Grafbase ⨯ Fauna

This examples hows how to create a GraphQL API using Grafbase Resolvers that can read and write data to a MySQL database hosted by Fauna &mdash; [Read the guide](https://grafbase.com/guides/build-and-deploy-a-graphql-api-to-the-edge-with-fauna)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-fauna grafbase-with-resolvers-fauna` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-fauna`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [Fauna](https://Fauna.com) database secret key.
5. Run `npx grafbase dev` to start local dev server
6. Open [Pathfinder](http://localhost:4000)
