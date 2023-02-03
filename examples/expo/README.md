# Create the base data

You can create the base data after creating the project by going to the playground and
executing this following mutation:

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
