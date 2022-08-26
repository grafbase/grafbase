import { ClientOptions, createClient } from 'solid-urql'

export const urqlClientBaseConfig: ClientOptions = {
  url: '/api/graphql',
  requestPolicy: 'cache-and-network'
}

export const urqlClient = createClient({
  ...urqlClientBaseConfig
})
