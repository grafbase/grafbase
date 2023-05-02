# Grafbase ⨯ Query Resolvers

This example shows how to create a new query that is added to your Grafbase GraphQL API &mdash; [Read the guide](https://grafbase.com/guides/working-with-query-resolvers-and-openai)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-query grafbase-with-resolvers-query` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-query`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [OpenAI API Key](https://openai.com/)
5. Run `npx grafbase dev` to start local dev server with your schema
6. Visit [http://localhost:4000](http://localhost:4000)
7. Ask OpenAI a question using GraphQL:

```graphql
{
  ask(prompt: "Who invented GraphQL?") {
    text
  }
}
```
