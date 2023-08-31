import { KvStore } from '../../grafbase-sdk/dist/shim.mjs'

export default {
  async fetch(request, env, _ctx) {
    const { pathname } = new URL(request.url)
    let key = pathname.split('/')[2]

    if (!key) {
      return new Response('missing key', { status: 404 })
    }

    const kvStore = new KvStore(env)
    const value = await kvStore.get(key)

    return new Response(value || 'not found')
  },
}
