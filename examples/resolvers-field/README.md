# Grafbase ⨯ Field Resolvers

This example shows how to extend a Grafbase Database `@model` with a custom field resolver &mdash; [Read the guide](https://grafbase.com/guides/working-with-field-resolvers-and-fetch)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-field grafbase-with-resolvers-field` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-field`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [OpenWeather API Key](https://openweathermap.org/api)
5. Run `npx grafbase dev` to start local dev server with your schema
6. Visit [http://localhost:4000](http://localhost:4000)
7. Populate your backend with a `Place`:

```graphql
mutation {
  placeCreate(
    input: {
      name: "Grand Hotel Central"
      location: { latitude: 41.3849706, longitude: 2.1755767 }
    }
  ) {
    place {
      id
    }
  }
}
```

8. Query for the place (using the `id` above) to get the current `weather`:

```graphql
{
  place(by: { id: "..." }) {
    name
    weather
  }
}
```
