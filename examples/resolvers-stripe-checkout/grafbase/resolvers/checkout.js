import Stripe from 'stripe'

const stripe = Stripe(process.env.STRIPE_SECRET_KEY)

export default async function Resolver(_, { input }) {
  const { lineItems } = input

  const { data } = await stripe.checkout.sessions.create({
    success_url: 'https://example.com/success',
    line_items: lineItems,
    mode: 'payment'
  })

  return { url: data.url }
}
