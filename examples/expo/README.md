# Grafbase ⨯ Expo (React Native)

[Join our Community](https://grafbase.com/community)

## Please note

This example uses Expo (React-Native) &mdash; [learn more](https://expo.dev/)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/expo grafbase-with-expo` to clone this example
2. Change directory into the new folder `cd grafbase-with-expo`
3. Run `npm install`
4. Run `cp .env.example .env` to copy the example `.env.example` file to `.env`
5. Open `.env` in your code editor and provide your Grafbase API endpoint and API key. Example should be fine for `npx grafbase dev`!
6. Run `npx grafbase@latest dev` in your terminal
7. Populate the backend with some `Emojis` entries using a GraphQL mutation:


```graphql
mutation {
  em1: emojiCreate(
    input: { char: "🍔", tags: [{ create: { text: "burguer" } }] }
  ) {
    __typename
  }
  em2: emojiCreate(
    input: { char: "❤️", tags: [{ create: { text: "heart" } }] }
  ) {
    __typename
  }
  em3: emojiCreate(
    input: { char: "🥷🏽", tags: [{ create: { text: "ninja" } }] }
  ) {
    __typename
  }
  em4: emojiCreate(
    input: { char: "💣", tags: [{ create: { text: "bomb" } }] }
  ) {
    __typename
  }
  em5: emojiCreate(
    input: { char: "🔥", tags: [{ create: { text: "fire" } }] }
  ) {
    __typename
  }
  em6: emojiCreate(
    input: { char: "📍", tags: [{ create: { text: "location" } }] }
  ) {
    __typename
  }
  em7: emojiCreate(
    input: { char: "💰", tags: [{ create: { text: "money" } }] }
  ) {
    __typename
  }
  em8: emojiCreate(
    input: { char: "📸", tags: [{ create: { text: "camera" } }] }
  ) {
    __typename
  }
}
```

8. In another terminal, run `npm start` to start the expo process.

9. Depending on your platform you may want to run the app on an [`Android`](https://docs.expo.dev/workflow/android-studio-emulator/) or [`iOS`](https://docs.expo.dev/workflow/ios-simulator/) emulator. 

## Learn More About Grafbase

To learn more about Grafbase, take a look at the following resources:

- [Grafbase](https://grafbase.com/) - learn about Grafbase features and API.

To learn more about Expo, take a look at the following resources:

- [Expo Documentation](https://docs.expo.dev/) - learn about Expo (React-Native).
