export class Header {
  name: string
  value: string

  constructor(name: string, value: string) {
    this.name = name
    this.value = value
  }

  public toString(): string {
    return `{ name: "${this.name}", value: "${this.value}"}`
  }
}

export interface OpenAPIParams {
  schema: string
  url?: string
}

export class PartialOpenAPI {
  schema: string
  apiUrl?: string
  headers: Header[]

  constructor(schema: string, url?: string) {
    this.schema = schema
    this.apiUrl = url
    this.headers = []
  }

  public header(name: string, value: string): PartialOpenAPI {
    this.headers.push(new Header(name, value))

    return this
  }

  finalize(namespace: string): OpenAPI {
    return new OpenAPI(namespace, this.schema, this.headers, this.apiUrl)
  }
}

export class OpenAPI {
  namespace: string
  schema: string
  apiUrl?: string
  headers: Header[]

  constructor(
    namespace: string,
    schema: string,
    headers: Header[],
    url?: string
  ) {
    this.namespace = namespace
    this.schema = schema
    this.apiUrl = url
    this.headers = headers
  }

  public toString(): string {
    const header = '  @openapi(\n'
    const namespace = this.namespace ? `    name: "${this.namespace}"\n` : ''
    const url = this.apiUrl ? `    url: "${this.apiUrl}"\n` : ''
    const schema = `    schema: "${this.schema}"\n`

    var headers = this.headers.map((header) => `      ${header}`).join('\n')
    headers = headers ? `    headers: [\n${headers}\n    ]\n` : ''

    const footer = '  )'

    return `${header}${namespace}${url}${schema}${headers}${footer}`
  }
}
