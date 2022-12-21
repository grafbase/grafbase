# Grafbase ⨯ Qwik

[Join our Community](https://grafbase.com/community)

This example uses the [Qwik](https://qwik.builder.io/docs/getting-started) web framework.

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/qwik grafbase-with-qwik` to clone this example
2. Change directory into the new folder `cd grafbase-with-qwik`
3. Run `cp .env.example .env`
4. Open `.env` in your code editor and provide your Grafbase API endpoint and API key. Example should be fine for `npx grafbase dev`!
5. Run `npm install`, or `yarn install` to install dependencies
6. Run `npx grafbase@latest dev` in your terminal
7. Populate the backend with some `Message` entries using a GraphQL mutation:

```graphql
mutation {
  plantCreate(
    input: { name: "pothos", description: "trailing marbled leaves" }
  ) {
    plant {
      id
      name
      description
    }
  }
}
```

8. In another terminal, run `npm start` and visit [`http://localhost:5173/`](http://localhost:5173/)

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

To learn more about Builder.io, take a look at the following resources:

- [Qwik City Documentation](https://qwik.builder.io/qwikcity/overview/) - learn about Qwik City.
- [Learn Qwik](https://qwik.builder.io/docs/overview/) - learn more about Qwik.

### Run on Codesandbox

[![Develop with CodeSandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/qwik)
