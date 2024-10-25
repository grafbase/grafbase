// @bun
var G
;((z) => {
  z.Debug = 'debug'
  z.Error = 'error'
  z.Info = 'info'
  z.Warn = 'warn'
})((G ||= {}))
var P = globalThis.fetch,
  Y = [],
  Z = []
if (import.meta.main) {
  const K = Bun.serve({ port: 0, fetch: (w) => O(w) })
  await Bun.write(Bun.stdout, `${K.port}\n`)
  for (let w of Object.values(G))
    globalThis.console[w] = (...C) => {
      Y.push({
        loggedAt: Date.now(),
        level: w,
        message: Array.from(C)
          .map((z) => JSON.stringify(z))
          .join(' '),
      })
    }
  ;(globalThis.console.log = globalThis.console.info),
    (globalThis.fetch = async (w, C) => {
      const z = new Request(w, C),
        W = Date.now(),
        j = await P(z),
        J = Date.now(),
        k = j.headers.get('content-type')?.split(';')[0].trim()
      let D = null
      switch (k) {
        case 'application/json':
        case 'text/plain':
        case 'text/html':
          D = await j.clone().text()
          break
      }
      const N = {
        loggedAt: Date.now(),
        url: z.url,
        method: z.method,
        statusCode: j.status,
        duration: J - W,
        contentType: k,
        body: D,
      }
      return Z.push(N), j
    })
  const O = (w) => {
    const C = new URL(w.url)
    switch (C.pathname) {
      case '/health':
        switch (w.method) {
          case 'GET':
            return new Response(JSON.stringify({ ready: !0 }), { headers: { ['content-type']: 'application/json' } })
          default:
            return new Response(X('method not allowed for /health'), { status: 405 })
        }
      case '/invoke':
        switch (w.method) {
          case 'POST':
            return $(w)
          default:
            return new Response(X('method not allowed for /invoke'), { status: 405 })
        }
      default:
        return new Response(X(`${C.pathname} not found`), { status: 404 })
    }
  }
}
var Q = (K) => {
    let O = ''
    for (let w of new Uint8Array(K)) O += String.fromCharCode(w)
    return btoa(O)
  },
  U = import.meta.require('${UDF_MAIN_FILE_PATH}').default,
  $ = async (K) => {
    ;(Y = []), (Z = [])
    let { parent: O, args: w, context: C, info: z, secrets: W } = await K.json()
    globalThis.process.env = Object.freeze({ ...globalThis.process.env, ...W })
    let j = null
    try {
      if (((C ??= {}), (j = U(O, w, C, z)), j instanceof Promise)) j = await j
      if (j instanceof Response)
        switch (j.headers.get('content-type')?.split(';')[0].trim()) {
          case 'application/json':
            j = await j.json()
            break
          case 'text/plain':
          case 'text/html':
            j = await j.text()
            break
          default:
            j = Q(await j.arrayBuffer())
            break
        }
      j = { Success: j }
    } catch (J) {
      if (J == null) j = { Error: 'nullish value thrown' }
      else if (J instanceof Error && J.name === 'GraphQLError')
        j = { GraphQLError: { message: J.message, extensions: J.extensions } }
      else j = { Error: J.toString() }
    }
    return new Response(JSON.stringify({ value: j, fetchRequests: Z, logEntries: Y }), {
      headers: { ['content-type']: 'application/json' },
    })
  },
  X = (K) => JSON.stringify({ value: { Error: K }, fetchRequests: [], logEntries: [] })
export { $ as invoke }
