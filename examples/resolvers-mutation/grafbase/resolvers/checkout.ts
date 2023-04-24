import Stripe from 'stripe'

export default async function Resolver(_, { input }) {
  const stripe = Stripe(process.env.STRIPE_SECRET_KEY)

  const { lineItems } = input

  const { url } = await stripe.checkout.sessions.create({
    success_url: 'https://example.com/success',
    line_items: lineItems,
    mode: 'payment'
  })

  return { url }
}
