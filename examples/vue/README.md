# Grafbase ⨯ Vue

[Join our Community](https://grafbase.com/community)

This example uses the [Vue](https://vuejs.org/) framework with [Vite](https://vitejs.dev/) and [Typescript](https://www.typescriptlang.org/). In production environments, you should switch to a supported [auth provider](https://grafbase.com/docs/auth/providers) using the `Authorization` header with requests to secure your data.

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/vue grafbase-with-vue` to clone this example
2. Change directory into the new folder `cd grafbase-with-vue`
3. Run `cp .env.example .env`
4. Open `.env` in your code editor and provide your Grafbase API endpoint and API key. Example should be fine for `npx grafbase@latest dev`!
5. Run `npm install`, or `yarn install` to install dependencies
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

8. In another terminal, run `npm run dev` and visit [`http://localhost:5174/`](http://localhost:5174/)

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

- To learn more about React, take a look at the [Vue Docs](https://vuejs.org/guide/introduction.html)

### Run on Codesandbox

[![Develop with CodeSandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/react)