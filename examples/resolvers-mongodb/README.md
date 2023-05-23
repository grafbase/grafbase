# Grafbase ⨯ MongoDB

This example shows how to create a Grafbase resolver that can execute read and write operations with a MongoDB database &mdash; [Read the guide](https://grafbase.com/guides/working-with-graphql-mongodb-data-api-and-edge-resolvers)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-mongodb grafbase-with-resolvers-mongodb` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-mongodb`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [MongoDB](https://account.mongodb.com/account/login) Data API URL and Key
5. Run `npx grafbase dev` to start local dev server with your schema
6. Open [Pathfinder](http://localhost:4000)
