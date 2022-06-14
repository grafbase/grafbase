import type { NextApiRequest, NextApiResponse } from 'next'

type Data = {
  data?: string
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {

  const response = await fetch(process.env.GRAFBASE_API_ENDPOINT as string, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization" : `Bearer ${process.env.GRAFBASE_API_KEY}`
    },
    body: JSON.stringify({
      query: `{
      __schema {
        types {
          name
        }
      }
    }`
    })
  }).then((data) => data.json())

  res.status(200).json(response)
}
