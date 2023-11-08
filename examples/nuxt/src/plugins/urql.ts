import urql, {
  createClient,
  dedupExchange,
  fetchExchange,
  ssrExchange
} from '@urql/vue'
import { devtoolsExchange } from '@urql/devtools'
import { defineNuxtPlugin, useRuntimeConfig } from '#imports'
import { cacheExchange } from '@/graphql/urql.exchange'

export default defineNuxtPlugin((nuxtApp) => {
  const { baseURL } = useRuntimeConfig()

  // Create SSR exchange
  const ssr = ssrExchange({
    isClient: process.client
  })

  // Extract SSR payload once app is rendered on the server
  if (process.server) {
    nuxtApp.hook('app:rendered', () => {
      if (nuxtApp.payload?.data) {
        nuxtApp.payload.data.urql = ssr.extractData()
      }
    })
  }

  // Restore SSR payload once app is created on the client
  if (process.client) {
    nuxtApp.hook('app:created', () => {
      if (nuxtApp.payload?.data) {
        ssr.restoreData(nuxtApp.payload.data.urql)
      }
    })
  }

  // Custom exchanges
  const exchanges = [
    dedupExchange,
    cacheExchange,
    ssr, // Add `ssr` in front of the `fetchExchange`
    fetchExchange
  ]

  // Devtools exchange
  if (nuxtApp._legacyContext?.isDev) {
    exchanges.unshift(devtoolsExchange)
  }

  // Instantiate urql client
  const client = createClient({
    url: baseURL + '/api/graphql',
    requestPolicy: 'cache-and-network',
    exchanges
  })

  nuxtApp.vueApp.use(urql, client)
})
