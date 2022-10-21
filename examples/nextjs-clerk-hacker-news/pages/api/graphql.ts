import type { NextApiRequest, NextApiResponse } from 'next'

type Data = {
  data?: string
}

type Error = {
  status?: number
  error?: string
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data | Error>
) {
  try {
    switch (req?.method?.toUpperCase()) {
      case 'POST':
        return res.status(200).json(
          await fetch(process.env.NEXT_PUBLIC_GRAFBASE_API_URL!, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'x-api-key': process.env.GRAFBASE_API_KEY!
            },
            body: JSON.stringify(req.body)
          }).then((response) => response.json())
        )
      default:
        return res.status(405).json({ status: 405 })
    }
  } catch (error) {
    return res.status(500).json({
      status: 500,
      error: JSON.stringify(error)
    })
  }
}
