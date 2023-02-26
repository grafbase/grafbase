# Grafbase ⨯ React

[Join our Community](https://grafbase.com/community)

This example uses the [React](https://reactjs.org/docs/getting-started.html) library.

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/react grafbase-with-react` to clone this example
2. Change directory into the new folder `cd grafbase-with-react`
3. Run `cp .env.example .env`
4. Open `.env` in your code editor and provide your Grafbase API endpoint and API key. Example should be fine for `npx grafbase dev`!
5. Run `npm install`, or `yarn install` to install dependencies
6. Run `npx grafbase@latest dev` in your terminal
7. Populate the backend with some `Message` entries using a GraphQL mutation:

```graphql
mutation {
  messageCreate(
    input: { author: "Grafbase Admin", message: "Grafbase is awesome!" }
  ) {
    message {
      id
    }
  }
}
```

8. In another terminal, run `npm run start` and visit [`http://localhost:8080/`](http://localhost:8080/)

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

- To learn more about React, take a look at the [React Beta Docs](https://beta.reactjs.org/)


### Run on Codesandbox

[![Develop with CodeSandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/react)
