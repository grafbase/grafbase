import { GraphQLClient } from 'graphql-request'

const endpoint = import.meta.env.VITE_GRAFBASE_API_URL
const apiKey = import.meta.env.VITE_GRAFBASE_API_KEY

export const grafbase = new GraphQLClient(endpoint, {
  headers: { 'x-api-key': apiKey }
})
