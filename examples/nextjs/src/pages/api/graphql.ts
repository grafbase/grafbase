import type { NextApiRequest, NextApiResponse } from 'next'
import serverSideFetch from 'utils/server-side-fetch'

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
        return res.status(200).json(await serverSideFetch(req.body))
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
