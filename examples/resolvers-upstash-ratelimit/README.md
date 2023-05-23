# Grafbase ⨯ Upstash

This examples shows how to create a Grafbase resolver rate limited by the Upstash ratelimit library on the edge &mdash; [Read the guide](https://grafbase.com/guides/working-with-resolvers-and-upstash-ratelimit)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-upstash-ratelimit grafbase-with-resolvers-upstash` to clone this example.
2. Change directory into the new folder `cd grafbase-with-resolvers-upstash`.
3. Run `cp grafbase/.env.example grafbase/.env`.
4. Open `grafbase/.env` in your code editor and provide the environment variables by following the related guide.
5. Run `npx grafbase dev` to start local dev server with your schema
6. Open [Pathfinder](http://localhost:4000)
