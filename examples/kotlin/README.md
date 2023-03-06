# Grafbase ⨯ Kotlin

[Join our Community](https://grafbase.com/community)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/kotlin grafbase-with-kotlin` to clone this example
2. Change directory into the new folder `cd grafbase-with-kotlin`
3. Run `npx grafbase@latest dev` in your terminal and go to [`http://localhost:4000`](http://localhost:4000)
4. Populate the backend with some `Post` entries using a GraphQL mutation:

```graphql
mutation {
  postCreate(
    input: {
      title: "Kotlin + GraphQL!"
      body: "Hello from Grafbase."
      comments: [
        { create: { message: "GraphQL is awesome!" }
        { create: { message: "Another comment from Grafbase" } }
      ]
    }
  ) {
    post {
      id
    }
  }
}
```

5. Open the project with Android Studio
6. Run the app!

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

To learn more about Kotlin, take a look at the following resources:

- [Kotlin Documentation](https://kotlinlang.org)

### Run on Codesandbox

[![Develop with Codesandbox](https://codesandbox.io/static/img/play-codesandbox.svg)](https://githubbox.com/grafbase/grafbase/tree/main/examples/kotlin)
