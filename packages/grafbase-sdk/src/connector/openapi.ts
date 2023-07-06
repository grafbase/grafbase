import { Header, Headers, HeaderGenerator } from './header'

export type OpenApiQueryNamingStrategy = 'OPERATION_ID' | 'SCHEMA_NAME'

export interface OpenApiTransformParams {
  queryNaming: OpenApiQueryNamingStrategy
}

export interface OpenAPIParams {
  schema: string
  url?: string
  transforms?: OpenApiTransformParams
  headers?: HeaderGenerator
}

export class OpenApiTransforms {
  private params: OpenApiTransformParams

  constructor(params: OpenApiTransformParams) {
    this.params = params
  }

  public toString(): string {
    return Object.entries(this.params)
      .map(([key, value]) => `${key}: ${value}`)
      .join(', ')
  }
}

export class PartialOpenAPI {
  private schema: string
  private apiUrl?: string
  private transforms?: OpenApiTransforms
  private headers: Header[]
  private introspectionHeaders: Header[]

  constructor(params: OpenAPIParams) {
    const headers = new Headers()

    if (params.headers) {
      params.headers(headers)
    }

    this.schema = params.schema
    this.apiUrl = params.url
    this.transforms = params.transforms
      ? new OpenApiTransforms(params.transforms)
      : undefined
    this.headers = headers.headers
    this.introspectionHeaders = headers.introspectionHeaders
  }

  finalize(namespace?: string): OpenAPI {
    return new OpenAPI(
      this.schema,
      this.headers,
      this.introspectionHeaders,
      this.transforms,
      this.apiUrl,
      namespace
    )
  }
}

export class OpenAPI {
  private namespace?: string
  private schema: string
  private apiUrl?: string
  private transforms?: OpenApiTransforms
  private headers: Header[]
  private introspectionHeaders: Header[]

  constructor(
    schema: string,
    headers: Header[],
    introspectionHeaders: Header[],
    transforms?: OpenApiTransforms,
    url?: string,
    namespace?: string
  ) {
    this.namespace = namespace
    this.schema = schema
    this.apiUrl = url
    this.transforms = transforms
    this.headers = headers
    this.introspectionHeaders = introspectionHeaders
  }

  public toString(): string {
    const header = '  @openapi(\n'
    const namespace = this.namespace
      ? `    namespace: "${this.namespace}"\n`
      : ''
    const url = this.apiUrl ? `    url: "${this.apiUrl}"\n` : ''
    const schema = `    schema: "${this.schema}"\n`

    const transforms = this.transforms
      ? `    transforms: { ${this.transforms} }\n`
      : ''

    var headers = this.headers.map((header) => `      ${header}`).join('\n')
    headers = headers ? `    headers: [\n${headers}\n    ]\n` : ''

    var introspectionHeaders = this.introspectionHeaders
      .map((header) => `      ${header}`)
      .join('\n')

    introspectionHeaders = introspectionHeaders
      ? `    introspectionHeaders: [\n${introspectionHeaders}\n    ]\n`
      : ''

    const footer = '  )'

    return `${header}${namespace}${url}${schema}${transforms}${headers}${introspectionHeaders}${footer}`
  }
}
