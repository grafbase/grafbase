# Grafbase ⨯ Eleventy (11ty)

[Join our Community](https://grafbase.com/community)

## Please note

This example uses the static site generator Eleventy (11ty) &mdash; [learn more](https://www.11ty.dev)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/eleventy grafbase-with-eleventy` to clone this example
2. Change directory into the new folder `cd grafbase-with-eleventy`
3. Run `npm install`
4. Run `cp .env.example .env` to copy the example `.env.example` file to `.env`
5. Open `.env` in your code editor and provide your Grafbase API endpoint and API key. Example should be fine for `npx grafbase dev`!
6. Run `npx grafbase@latest dev` in your terminal
7. Populate the backend with some `Message` entries using a GraphQL mutation:

```graphql
mutation {
  messageCreate(
    input: { author: "Grafbase Admin", body: "Grafbase is awesome!" }
  ) {
    message {
      id
    }
  }
}
```

8. In another terminal, run `npm start` and visit [`http://localhost:8080`](http://localhost:8080)

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

To learn more about Next.js, take a look at the following resources:

- [11ty Documentation](https://www.11ty.dev/docs/) - learn about Eleventy (11ty).

### Run on Codesandbox

[![Develop with Codesandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/eleventy)
