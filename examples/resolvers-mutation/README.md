# Grafbase ⨯ Mutation Resolvers

This example shows how to create a new mutation that is added to your Grafbase GraphQL API &mdash; [Read the guide](https://grafbase.com/guides/working-with-mutation-resolvers-and-stripe-checkout)

## Getting Started

1. Run `npx degit grafbase/grafbase/examples/resolvers-mutation grafbase-with-resolvers-mutation` to clone this example
2. Change directory into the new folder `cd grafbase-with-resolvers-mutation`
3. Run `cp grafbase/.env.example grafbase/.env`
4. Open `grafbase/.env` in your code editor and provide your [Stripe Secret Key](https://dashboard.stripe.com)
5. Run `npx grafbase dev` to start local dev server with your schema
6. Visit [http://localhost:4000](http://localhost:4000)
7. Add a product and price via the [Stripe Dashboard](https://dashboard.stripe.com)
8. Create a new Stripe Checkout Session for the Product/Price created above:

```graphql
mutation {
  checkout(input: {
    lineItems: [
      {
        price: 'price_1Mt5h0D1CEk8AY6BaIVyRJRN',
        quantity: 1
      }
    ]
  }) {
    url
  }
}
```
