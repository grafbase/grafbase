import Stripe from 'stripe'

// eslint-disable-next-line turbo/no-undeclared-env-vars
const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!, {
  apiVersion: '2022-11-15',
  typescript: true
})

export default async function Resolver(_, { input }) {
  const { lineItems } = input

  const data = await stripe.checkout.sessions.create({
    success_url: 'https://example.com/success',
    line_items: lineItems,
    mode: 'payment'
  })

  return { url: data.url }
}
