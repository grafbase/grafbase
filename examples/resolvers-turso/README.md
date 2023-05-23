# Grafbase ⨯ Turso

This examples hows how to create a Grafbase resolver that can execute read and write operations with a Turso database &mdash; [Read the guide](https://grafbase.com/guides/working-with-graphql-and-turso-using-edge-resolvers)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-turso grafbase-with-resolvers-turso` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-turso`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [Turso](https://turso.tech/) Database URL and Access Token
5. Run `npx grafbase dev` to start local dev server with your schema
6. Open [Pathfinder](http://localhost:4000)
