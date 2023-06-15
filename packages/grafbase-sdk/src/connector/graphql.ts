import { Header, Headers, HeaderGenerator } from './header'

export interface GraphQLParams {
  url: string
  headers?: HeaderGenerator
}

export class PartialGraphQLAPI {
  private apiUrl: string
  private headers: Header[]
  private introspectionHeaders: Header[]

  constructor(params: GraphQLParams) {
    const headers = new Headers()

    if (params.headers) {
      params.headers(headers)
    }

    this.apiUrl = params.url
    this.headers = headers.headers
    this.introspectionHeaders = headers.introspectionHeaders
  }

  finalize(namespace?: string): GraphQLAPI {
    return new GraphQLAPI(
      this.apiUrl,
      this.headers,
      this.introspectionHeaders,
      namespace
    )
  }
}

export class GraphQLAPI {
  private namespace?: string
  private url: string
  private headers: Header[]
  private introspectionHeaders: Header[]

  constructor(
    url: string,
    headers: Header[],
    introspectionHeaders: Header[],
    namespace?: string
  ) {
    this.namespace = namespace
    this.url = url
    this.headers = headers
    this.introspectionHeaders = introspectionHeaders
  }

  public toString(): string {
    const header = '  @graphql(\n'
    const namespace = this.namespace ? `    namespace: "${this.namespace}"\n` : ''
    const url = this.url ? `    url: "${this.url}"\n` : ''

    var headers = this.headers.map((header) => `      ${header}`).join('\n')
    headers = headers ? `    headers: [\n${headers}\n    ]\n` : ''

    var introspectionHeaders = this.introspectionHeaders
      .map((header) => `      ${header}`)
      .join('\n')

    introspectionHeaders = introspectionHeaders
      ? `    introspectionHeaders: [\n${introspectionHeaders}\n    ]\n`
      : ''

    const footer = '  )'

    return `${header}${namespace}${url}${headers}${introspectionHeaders}${footer}`
  }
}
