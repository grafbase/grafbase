import { cache } from 'react'

export const gql = cache(async (query: string) => {
  const apiUrl = process.env.GRAFBASE_API_URL || 'http://127.0.0.1:4000/graphql'
  const apiKey = process.env.GRAFBASE_API_KEY || 'letmein'

  const res = await fetch(apiUrl, {
    headers: {
      'x-api-key': apiKey
    },
    method: 'POST',
    body: JSON.stringify({
      query
    }),
    next: {
      tags: [query]
    }
  })

  const { data } = await res.json()

  return data
})
