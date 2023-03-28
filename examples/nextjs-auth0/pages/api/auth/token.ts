import { NextApiRequest, NextApiResponse } from 'next'
import { withApiAuthRequired, getAccessToken } from '@auth0/nextjs-auth0'

async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { accessToken: token } = await getAccessToken(req, res, {
    authorizationParams: {
      audience: 'https://grafbase.com',
      issuerBaseURL: 'https://grafbase-examples.eu.auth0.com/',
      tokenSigningAlg: 'RS256'
    }
  })

  res.json({ token })
}

export default withApiAuthRequired(handler)
