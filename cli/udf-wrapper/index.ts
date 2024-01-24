/// @ts-expect-error set individually for each UDF
import udf from '${UDF_MAIN_FILE_PATH}'
// import { IncomingMessage, ServerResponse, createServer } from 'http'
// import { Readable } from 'stream'
import { KVNamespace } from '@miniflare/kv'
import { MemoryStorage } from '@miniflare/storage-memory'

interface LogEntry {
  loggedAt: number
  level: string
  message: string
}

interface FetchRequest {
  loggedAt: number
  url: string
  method: string
  statusCode: number
  duration: number
  contentType?: string
  body: string | null
}

interface UdfRequestPayload {
  info: unknown
  parent: unknown
  context: { kv?: KVNamespace }
  args: unknown
}

// type NodeResponse = ServerResponse<IncomingMessage> & { req: IncomingMessage }

// type Headers = Record<string, string | string[]>

enum HttpMethod {
  Get = 'GET',
  Post = 'POST',
}

enum HttpStatus {
  NotFound = 404,
  MethodNotAllowed = 405,
}

enum LogLevel {
  Debug = 'debug',
  Error = 'error',
  Info = 'info',
  Warn = 'warn',
}

enum MimeType {
  ApplicationJson = 'application/json',
  TextHtml = 'text/html',
  TextPlain = 'text/plain',
}

enum Route {
  Health = '/health',
  Invoke = '/invoke',
}

enum ErrorType {
  GraphQL = 'GraphQLError',
}

enum Header {
  ContentType = 'content-type',
}

// enum Duplex {
//   Half = 'half',
// }

// enum StreamEvent {
//   Data = 'data',
//   End = 'end',
// }

const PORT = 0
// const HOST = '127.0.0.1'
// const DUMMY_HOST = 'https://grafbase-cli'
const MIME_PROPERTY_SEPARATOR = ';'

const originalFetch = globalThis.fetch

let logEntries: Array<LogEntry> = []
let fetchRequests: Array<FetchRequest> = []

// // Node.js:

// const server = createServer((request, response) => {
//   const routerResponse = router(nodeRequestToWeb(request))

//   if (routerResponse instanceof Promise) {
//     routerResponse.then((routerResponse) => webResponseToNode(routerResponse, response))
//   } else {
//     webResponseToNode(routerResponse, response)
//   }
// })

// server.listen(PORT, HOST, () => {
//   // @ts-expect-error incorrectly typed for the `.port` property, `.address()` must exist at this point
//   const port = server.address().port
//   originalConsoleLog(port)
// })

// const nodeRequestToWeb = (request: IncomingMessage) =>
//   new Request(`${DUMMY_HOST}${request.url}`, {
//     method: request.method,
//     // the cast here is likely required because of node fetch still being experimental
//     headers: request.headers as Headers,
//     body: request.method === HttpMethod.Post ? Readable.toWeb(request) : undefined,
//     duplex: Duplex.Half,
//   })

// const webResponseToNode = (webResponse: Response, response: NodeResponse) => {
//   webResponse.headers.forEach((value, key) => response.setHeader(key, value))
//   response.statusMessage = webResponse.statusText
//   response.statusCode = webResponse.status
//   if (webResponse.body !== null) {
//     Readable.fromWeb(webResponse.body)
//       .on(StreamEvent.Data, (chunk) => response.write(chunk))
//       .on(StreamEvent.End, () => response.end())
//   } else {
//     response.end()
//   }
// }

// Deno:

// Deno.serve(
//   { port: PORT, onListen: (path: { hostname: string; port: number }) => originalConsoleLog(path.port) },
//   (request: Request) => router(request),
// )

// Bun:
if (import.meta.main) {
  const server = Bun.serve({
    port: PORT,
    fetch: (request: Request) => router(request),
  })

  // @ts-expect-error incorrect typing
  await Bun.write(Bun.stdout, `${server.port}\n`)

  // patches console.* to return the logs in the response
  for (const level of Object.values(LogLevel)) {
    globalThis.console[level] = (...message: unknown[]) => {
      logEntries.push({
        loggedAt: Date.now(),
        level,
        message: Array.from(message)
          .map((message) => JSON.stringify(message))
          .join(' '),
      })
    }
  }

  globalThis.console.log = globalThis.console.info

  // Monkey patch `fetch()` calls from custom resolvers
  // to allow for fully introspected logging of all HTTP requests.
  globalThis.fetch = async (input: string | URL | Request, init?: RequestInit) => {
    const request = new Request(input as Request /* incorrect typing for Bun */, init)

    const startTime = Date.now()
    const response = await originalFetch(request)
    const endTime = Date.now()

    const contentType = response.headers.get(Header.ContentType)?.split(MIME_PROPERTY_SEPARATOR)[0].trim()

    let body: string | null = null

    switch (contentType) {
      case MimeType.ApplicationJson:
      case MimeType.TextPlain:
      case MimeType.TextHtml:
        body = await response.clone().text()
        break
    }

    const fetchRequest: FetchRequest = {
      loggedAt: Date.now(),
      url: request.url,
      method: request.method,
      statusCode: response.status,
      duration: endTime - startTime,
      contentType,
      body,
    }

    fetchRequests.push(fetchRequest)

    return response
  }

  const router = (request: Request) => {
    const url = new URL(request.url)
    switch (url.pathname) {
      case Route.Health:
        switch (request.method) {
          case HttpMethod.Get:
            return new Response(JSON.stringify({ ready: true }), {
              headers: { [Header.ContentType]: MimeType.ApplicationJson },
            })
          default:
            return new Response(toErrorResponse(`method not allowed for ${Route.Health}`), {
              status: HttpStatus.MethodNotAllowed,
            })
        }
      case Route.Invoke:
        switch (request.method) {
          case HttpMethod.Post:
            return invoke(request)
          default:
            return new Response(toErrorResponse(`method not allowed for ${Route.Invoke}`), {
              status: HttpStatus.MethodNotAllowed,
            })
        }
      default:
        return new Response(toErrorResponse(`${url.pathname} not found`), {
          status: HttpStatus.NotFound,
        })
    }
  }
}

const arrayBufferToBase64 = (buffer: ArrayBuffer) => {
  let binaryString = ''
  for (const byte of new Uint8Array(buffer)) {
    binaryString += String.fromCharCode(byte)
  }
  return btoa(binaryString)
}

export const invoke = async (request: Request) => {
  logEntries = []
  fetchRequests = []

  const { parent, args, context, info } = (await request.json()) as UdfRequestPayload

  let returnValue: unknown = null

  try {
    context.kv = new KVNamespace(new MemoryStorage())

    returnValue = udf(parent, args, context, info)

    if (returnValue instanceof Promise) {
      returnValue = await returnValue
    }

    if (returnValue instanceof Response) {
      const contentType = returnValue.headers.get(Header.ContentType)?.split(MIME_PROPERTY_SEPARATOR)[0].trim()
      switch (contentType) {
        case MimeType.ApplicationJson:
          returnValue = await returnValue.json()
          break
        case MimeType.TextPlain:
        case MimeType.TextHtml:
          returnValue = await returnValue.text()
          break
        default:
          returnValue = arrayBufferToBase64(await returnValue.arrayBuffer())
          break
      }
    }

    returnValue = { Success: returnValue }
  } catch (error: unknown) {
    if (error == null) {
      returnValue = { Error: 'nullish value thrown' }
    } else {
      if (error instanceof Error && error.name === ErrorType.GraphQL) {
        returnValue = {
          GraphQLError: {
            message: error.message,
            // @ts-expect-error this is a part of GraphQLError
            extensions: error.extensions,
          },
        }
      } else {
        returnValue = { Error: error.toString() }
      }
    }
  }

  return new Response(JSON.stringify({ value: returnValue, fetchRequests, logEntries }), {
    headers: { [Header.ContentType]: MimeType.ApplicationJson },
  })
}

const toErrorResponse = (error: string) =>
  JSON.stringify({
    value: { Error: error },
    fetchRequests: [],
    logEntries: [],
  })
