import { GraphQLClient } from 'graphql-request'

let client: GraphQLClient

declare global {
  var __client: GraphQLClient | undefined
}

const createClient = () => {
  const { GRAFBASE_API_URL, GRAFBASE_API_KEY } = process.env

  if (!GRAFBASE_API_URL || !GRAFBASE_API_KEY) {
    throw new Error('GRAFBASE_API_URL and GRAFBASE_API_KEY must be set')
  }

  return new GraphQLClient(GRAFBASE_API_URL, {
    headers: { 'x-api-key': GRAFBASE_API_KEY }
  })
}

// this is needed because in development we don't want to restart
// the server with every change, but we want to make sure we don't
// create a new connection to the API with every change either.
if (process.env.NODE_ENV === 'production') {
  client = createClient()
} else {
  if (!global.__client) {
    global.__client = createClient()
  }
  client = global.__client
}

export { client }
