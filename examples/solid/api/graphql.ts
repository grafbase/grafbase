export const config = {
  runtime: 'experimental-edge'
}

export default async (req: Request) => {
  try {
    switch (req.method.toUpperCase()) {
      case 'POST':
        const res = await serverSideFetch(req)
        return response(200, res.body)
      default:
        return response(405)
    }
  } catch (error) {
    return response(500, JSON.stringify(error))
  }
}

const response = (status: number, body?: BodyInit | null) => {
  return new Response(body, {
    status,
    headers: {
      'content-type': 'application/json'
    }
  })
}

const serverSideFetch = async (req: Request) => {
  const { GRAFBASE_API_URL, GRAFBASE_API_KEY } = process.env
  if (!GRAFBASE_API_URL || !GRAFBASE_API_KEY) throw new Error('Setup env vars')
  return await fetch(GRAFBASE_API_URL, {
    method: 'POST',
    body: req.body,
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': GRAFBASE_API_KEY
    }
  })
}
