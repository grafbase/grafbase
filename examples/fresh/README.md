# Grafbase ⨯ Fresh

[Join our Community](https://grafbase.com/community)

## Please note

This example uses the next-gen web framework Fresh &mdash; [learn more](https://fresh.deno.dev/)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/fresh grafbase-with-fresh` to clone this example
2. Change directory into the new folder `cd grafbase-with-fresh`
3. Run `cp .env.example .env` to copy the example `.env.example` file to `.env`
4. Open `.env` in your code editor and provide your Grafbase API endpoint and API key. Example should be fine for `npx grafbase dev`!
5. Run `npx grafbase@latest dev` in your terminal
6. Populate the backend with some `Post` entries using a GraphQL mutation:

```graphql
mutation {
  postCreate(input: { title: "Hello Fresh", slug: "hello-fresh" }) {
    post {
      id
    }
  }
}
```

6. In another terminal, run `deno task start` and visit [`http://localhost:8000`](http://localhost:8000)

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

To learn more about Next.js, take a look at the following resources:

- [Fresh Documentation](https://fresh.deno.dev/) - learn about Fresh.
- [Learn Deno](https://deno.land/) - learn more about Deno.

### Run on Codesandbox

[![Develop with Codesandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/fresh)
