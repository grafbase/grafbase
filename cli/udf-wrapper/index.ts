//import udf from '${UDF_MAIN_FILE_PATH}'
import { createServer } from 'http'
import { KVNamespace } from '@miniflare/kv'
import { MemoryStorage } from '@miniflare/storage-memory'
import { Readable } from 'stream'
import { ReadableStream } from 'stream/web'

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

enum Duplex {
  Half = 'half',
}

enum StreamEvent {
  Data = 'data',
  End = 'end',
}

const PORT = 0
const HOST = '127.0.0.1'
const DUMMY_HOST = 'https://grafbase-cli'
const MIME_PROPERTY_SEPARATOR = ';'
const STDOUT = Symbol()

const server = createServer((request, response) => {
  router(
    new Request(`${DUMMY_HOST}${request.url}`, {
      method: request.method,
      // the cast here is likely required because of node fetch still being experimental
      headers: request.headers as Record<string, string>,
      body: Readable.toWeb(request),
      // @ts-expect-error https://github.com/node-fetch/node-fetch/issues/1769
      duplex: Duplex.Half,
    }),
  ).then((udfResponse) => {
    udfResponse.headers.forEach((value, key) => response.setHeader(key, value))
    response.statusMessage = udfResponse.statusText
    response.statusCode = udfResponse.status
    Readable.fromWeb(udfResponse.body as ReadableStream<Uint8Array>)
      .on(StreamEvent.Data, (data) => response.write(data))
      .on(StreamEvent.End, () => response.end())
  })
})

server.listen(PORT, HOST, () => {
  // @ts-expect-error incorrectly typed
  const port = server.address().port
  globalThis[STDOUT](port)
})

const arrayBufferToBase64 = (buffer: ArrayBuffer) => {
  let binaryString = ''
  const byteArray = new Uint8Array(buffer)
  for (const byte of byteArray) {
    binaryString += String.fromCharCode(byte)
  }
  return btoa(binaryString)
}

// FIXME: testing only, remove
const udf = async (_parent: unknown, _args: unknown, context: { kv: KVNamespace }, _info: unknown) => {
  await context.kv.put('test', '1')
  console.log(await context.kv.get('test'))
  await fetch('https://example.com').then((response) => response.text())
  return { hello: 'world' }
}

let logEntries: LogEntry[] = []

// allows the wrapper to output the port
globalThis[STDOUT] = console.log

for (const level of [LogLevel.Debug, LogLevel.Error, LogLevel.Info, LogLevel.Warn]) {
  globalThis.console[level] = function (...message: unknown[]) {
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
let fetchRequests: FetchRequest[] = []

const originalFetch = globalThis.fetch

globalThis.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
  const request = new Request(input, init)

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

const router = async (request: Request) => {
  const url = new URL(request.url)
  switch (url.pathname) {
    case Route.Health: {
      switch (request.method) {
        case HttpMethod.Get: {
          return new Response(JSON.stringify({ ready: true }), {
            headers: {
              [Header.ContentType]: MimeType.ApplicationJson,
            },
          })
        }
        default: {
          return new Response(`method not allowed for ${Route.Health}`, { status: HttpStatus.MethodNotAllowed })
        }
      }
    }
    case Route.Invoke: {
      switch (request.method) {
        case HttpMethod.Post: {
          return await invoke(request)
        }
        default: {
          return new Response(`method not allowed for ${Route.Invoke}`, { status: HttpStatus.MethodNotAllowed })
        }
      }
    }
    default: {
      return new Response(`${url.pathname} not found`, { status: HttpStatus.NotFound })
    }
  }
}

const invoke = async (request: Request) => {
  logEntries = []
  fetchRequests = []

  const { parent, args, context, info } = await request.json()

  let returnValue: unknown = null

  try {
    if (context) {
      context.kv = new KVNamespace(new MemoryStorage())
    }

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

    returnValue = {
      Success: returnValue,
    }
  } catch (error: unknown) {
    if (error == null) {
      returnValue = {
        Error: 'nullish value thrown',
      }
    } else {
      if (error instanceof Error && error.name === ErrorType.GraphQL) {
        returnValue = {
          GraphQLError: {
            message: error.message,
            // @ts-expect-error
            extensions: error.extensions,
          },
        }
      } else {
        returnValue = {
          Error: error.toString(),
        }
      }
    }
  }

  const jsonResponse = { value: returnValue, fetchRequests, logEntries: logEntries }

  return new Response(JSON.stringify(jsonResponse), {
    headers: {
      [Header.ContentType]: MimeType.ApplicationJson,
    },
  })
}
