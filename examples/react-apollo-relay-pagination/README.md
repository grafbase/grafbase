# Grafbase ⨯ Apollo Client (Relay Pagination)

[Join our Community](https://grafbase.com/community)

## Please note

This example doesn't implement any authentication provider. You will need to add one to use this example in production &mdash; [learn more](http://grafbase.com/guides).

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/react-apollo-relay-pagination grafbase-with-react-apollo-relay-pagination` to clone this example
2. Change directory into the new folder `cd grafbase-with-react-apollo-relay-pagination`
3. Run `npm install`
4. Run `npx grafbase@latest dev` in your terminal
5. Populate the backend with some `Post` entries using a GraphQL mutation (make sure to repeat this a few times with different `title` values):

```graphql
mutation {
  postCreate(input: { title: "Grafbase" }) {
    post {
      id
    }
  }
}
```

6. In another terminal, run `npm start` and visit [`http://localhost:3000`](http://localhost:3000)

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) &mdash; learn about Grafbase features and API.
- [Pagination](https://grafbase.com/docs/reference/pagination) &mdash; learn about Grafbase pagination API.

To learn more about Apollo Client, take a look at the following resources:

- [Apollo Client Documentation](https://www.apollographql.com/docs) - learn about Apollo Client.
- [Relay style pagination](https://www.apollographql.com/docs/react/pagination/cursor-based#relay-style-cursor-pagination) &mdash; learn about Relay style pagination.

### Run on Codesandbox

[![Develop with Codesandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/react-apollo-relay-pagination)
