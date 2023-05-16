export class Header {
  name: string
  value: string

  constructor(name: string, value: string) {
    this.name = name
    this.value = value
  }

  public toString(): string {
    return `{ name: "${this.name}", value: "${this.value}" }`
  }
}

export class Headers {
  headers: Header[]
  introspectionHeaders: Header[]

  constructor() {
    this.headers = []
    this.introspectionHeaders = []
  }

  /**
  * Creates a header used in client and introspection requests.
  *
  * @param name - The name of the header
  * @param value - The value of the header
  */
  public static(name: string, value: string) {
    this.headers.push(new Header(name, value))
  }

  /**
  * Creates a header used only in introspection requests.
  *
  * @param name - The name of the header
  * @param value - The value of the header
  */
  public introspection(name: string, value: string) {
    this.introspectionHeaders.push(new Header(name, value))
  }
}

export type HeaderGenerator = (headers: Headers) => any

export interface OpenAPIParams {
  schema: string
  url?: string
  headers?: HeaderGenerator
}

export class PartialOpenAPI {
  schema: string
  apiUrl?: string
  headers: Header[]
  introspectionHeaders: Header[]
  
  constructor(params: OpenAPIParams) {
    const headers = new Headers()

    if (params.headers) {
      params.headers(headers)
    }
    
    this.schema = params.schema
    this.apiUrl = params.url
    this.headers = headers.headers
    this.introspectionHeaders = headers.introspectionHeaders
  }

  finalize(namespace: string): OpenAPI {
    return new OpenAPI(namespace, this.schema, this.headers, this.introspectionHeaders, this.apiUrl)
  }
}

export class OpenAPI {
  namespace: string
  schema: string
  apiUrl?: string
  headers: Header[]
  introspectionHeaders: Header[]

  constructor(namespace: string, schema: string, headers: Header[], introspectionHeaders: Header[], url?: string) {
    this.namespace = namespace
    this.schema = schema
    this.apiUrl = url
    this.headers = headers
    this.introspectionHeaders = introspectionHeaders
  }

  public toString(): string {
    const header = "  @openapi(\n"
    const namespace = this.namespace ? `    name: "${this.namespace}"\n` : ""
    const url = this.apiUrl ? `    url: "${this.apiUrl}"\n` : ""
    const schema = `    schema: "${this.schema}"\n`

    var headers = this.headers.map((header) => `      ${header}`).join("\n")
    headers = headers ? `    headers: [\n${headers}\n    ]\n`: ""

    var introspectionHeaders = this.introspectionHeaders.map((header) => `      ${header}`).join("\n")
    introspectionHeaders = headers ? `    introspectionHeaders: [\n${introspectionHeaders}\n    ]\n`: ""

    const footer = "  )"

    return `${header}${namespace}${url}${schema}${headers}${introspectionHeaders}${footer}`
  }
}