import { Header, PartialHeaderGenerator, PartialHeaders } from './header'

export interface GraphQLParams {
  url: string
  headers?: PartialHeaderGenerator
}

export class PartialGraphQLAPI {
  apiUrl: string
  headers: Header[]

  constructor(params: GraphQLParams) {
    const headers = new PartialHeaders()

    if (params.headers) {
      params.headers(headers)
    }

    this.apiUrl = params.url
    this.headers = headers.headers
  }

  finalize(namespace: string): GraphQLAPI {
    return new GraphQLAPI(namespace, this.apiUrl, this.headers)
  }
}

export class GraphQLAPI {
  namespace: string
  url: string
  headers: Header[]

  constructor(namespace: string, url: string, headers: Header[]) {
    this.namespace = namespace
    this.url = url
    this.headers = headers
  }

  public toString(): string {
    const header = '  @graphql(\n'
    const namespace = this.namespace ? `    name: "${this.namespace}"\n` : ''
    const url = this.url ? `    url: "${this.url}"\n` : ''

    var headers = this.headers.map((header) => `      ${header}`).join('\n')
    headers = headers ? `    headers: [\n${headers}\n    ]\n` : ''

    const footer = '  )'

    return `${header}${namespace}${url}${headers}${footer}`
  }
}
