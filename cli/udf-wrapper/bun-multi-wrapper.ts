import { parseArgs } from "util";

interface LogEntry {
  loggedAt: number;
  level: string;
  message: string;
}

interface FetchRequest {
  loggedAt: number;
  url: string;
  method: string;
  statusCode: number;
  duration: number;
  contentType?: string;
  body: string | null;
}

enum HttpMethod {
  Get = "GET",
  Post = "POST",
}

enum HttpStatus {
  NotFound = 404,
  MethodNotAllowed = 405,
}

enum LogLevel {
  Debug = "debug",
  Error = "error",
  Info = "info",
  Warn = "warn",
}

enum MimeType {
  ApplicationJson = "application/json",
  TextHtml = "text/html",
  TextPlain = "text/plain",
}

enum Route {
  Health = "health",
  Invoke = "invoke",
}

enum Header {
  ContentType = "content-type",
}

const PORT = 0;
const MIME_PROPERTY_SEPARATOR = ";";

const originalFetch = globalThis.fetch;

let logEntries: Array<LogEntry> = [];
let fetchRequests: Array<FetchRequest> = [];

const { positionals } = parseArgs({
  args: Bun.argv,
  strict: true,
  allowPositionals: true,
});

positionals.shift();
positionals.shift();

const toUdfKey = (name: string, kind: string) => `${name}${kind}`;

const udfs: { [key: string]: (request: Request) => Promise<Response> } =
  Object.fromEntries(
    await Promise.all(
      positionals.map(async (script: string) => {
        const [name, kind, ...pathParts] = script.split(":");
        const path = pathParts.join(":");
        const udf = await import(path);
        return [toUdfKey(name, kind), udf.invoke];
      })
    )
  );

const server = Bun.serve({
  port: PORT,
  fetch:  (request: Request) => {
    const url = new URL(request.url);
    if (url.pathname === "/health") {
      return router(request, Route.Health);
    }
    const [, kind, name, action] = url.pathname.split('/', 4);
    if ([kind, name, action].some(part => part == null) || udfs[toUdfKey(name, kind)] == null) {
      const url = new URL(request.url);
      return new Response(toErrorResponse(`${url.pathname} not found`), {
        status: HttpStatus.NotFound,
      });
    }
    return router(request, action, udfs[toUdfKey(name, kind)]);
  },
});

// @ts-expect-error incorrect typing
await Bun.write(Bun.stdout, `${server.port}\n`);

// patches console.* to return the logs in the response
for (const level of Object.values(LogLevel)) {
  globalThis.console[level] = (...message: unknown[]) => {
    logEntries.push({
      loggedAt: Date.now(),
      level,
      message: Array.from(message)
        .map((message) => JSON.stringify(message))
        .join(" "),
    });
  };
}

globalThis.console.log = globalThis.console.info;

// Monkey patch `fetch()` calls from custom resolvers
// to allow for fully introspected logging of all HTTP requests.
globalThis.fetch = async (
  input: string | URL | Request,
  init?: RequestInit
) => {
  const request = new Request(
    input as Request /* incorrect typing for Bun */,
    init
  );

  const startTime = Date.now();
  const response = await originalFetch(request);
  const endTime = Date.now();

  const contentType = response.headers
    .get(Header.ContentType)
    ?.split(MIME_PROPERTY_SEPARATOR)[0]
    .trim();

  let body: string | null = null;

  switch (contentType) {
    case MimeType.ApplicationJson:
    case MimeType.TextPlain:
    case MimeType.TextHtml:
      body = await response.clone().text();
      break;
  }

  const fetchRequest: FetchRequest = {
    loggedAt: Date.now(),
    url: request.url,
    method: request.method,
    statusCode: response.status,
    duration: endTime - startTime,
    contentType,
    body,
  };

  fetchRequests.push(fetchRequest);

  return response;
};

const toErrorResponse = (error: string) =>
  JSON.stringify({
    value: { Error: error },
    fetchRequests: [],
    logEntries: [],
  });

const router = (
  request: Request,
  action: string,
  invoke?: (request: Request) => Promise<Response>
) => {
  switch (action) {
    case Route.Health:
      switch (request.method) {
        case HttpMethod.Get:
          return new Response(JSON.stringify({ ready: true }), {
            headers: { [Header.ContentType]: MimeType.ApplicationJson },
          });
        default:
          return new Response(
            toErrorResponse(`method not allowed for ${Route.Health}`),
            {
              status: HttpStatus.MethodNotAllowed,
            }
          );
      }
    case Route.Invoke:
      switch (request.method) {
        case HttpMethod.Post:
          return invoke?.(request);
        default:
          return new Response(
            toErrorResponse(`method not allowed for ${Route.Invoke}`),
            {
              status: HttpStatus.MethodNotAllowed,
            }
          );
      }
    default:
      return new Response(toErrorResponse(`${action} not found`), {
        status: HttpStatus.NotFound,
      });
  }
};
