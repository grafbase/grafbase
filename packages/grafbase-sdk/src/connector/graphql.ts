import { Header, Headers, HeaderGenerator } from './header'
import {
  SchemaTransform,
  SchemaTransforms,
  TransformsGenerator
} from './transforms'

export interface GraphQLParams {
  url: string
  headers?: HeaderGenerator
  transforms?: TransformsGenerator
}

export class PartialGraphQLAPI {
  private name: string
  private apiUrl: string
  private headers: Header[]
  private introspectionHeaders: Header[]
  private transforms: SchemaTransform[]

  constructor(name: string, params: GraphQLParams) {
    const headers = new Headers()

    if (params.headers) {
      params.headers(headers)
    }

    const transforms = new SchemaTransforms()
    if (params.transforms) {
      params.transforms(transforms)
    }

    this.name = name
    this.apiUrl = params.url
    this.headers = headers.headers
    this.introspectionHeaders = headers.introspectionHeaders
    this.transforms = transforms.transforms
  }

  finalize(namespace?: string): GraphQLAPI {
    return new GraphQLAPI(
      this.name,
      this.apiUrl,
      this.headers,
      this.introspectionHeaders,
      this.transforms,
      namespace
    )
  }
}

export class GraphQLAPI {
  private name: string
  private namespace?: string
  private url: string
  private headers: Header[]
  private introspectionHeaders: Header[]
  private transforms: SchemaTransform[]

  constructor(
    name: string,
    url: string,
    headers: Header[],
    introspectionHeaders: Header[],
    transforms: SchemaTransform[],
    namespace?: string
  ) {
    this.name = name
    this.namespace = namespace
    this.url = url
    this.headers = headers
    this.introspectionHeaders = introspectionHeaders
    this.transforms = transforms
  }

  public toString(): string {
    const header = '  @graphql(\n'
    const name = `    name: "${this.name}"\n`

    const namespace = this.namespace
      ? `    namespace: "${this.namespace}"\n`
      : ''

    const url = this.url ? `    url: "${this.url}"\n` : ''

    let headers = this.headers.map((header) => `      ${header}`).join('\n')
    headers = headers ? `    headers: [\n${headers}\n    ]\n` : ''

    let introspectionHeaders = this.introspectionHeaders
      .map((header) => `      ${header}`)
      .join('\n')

    introspectionHeaders = introspectionHeaders
      ? `    introspectionHeaders: [\n${introspectionHeaders}\n    ]\n`
      : ''

    let transforms = this.transforms
      .map((transform) => `      ${transform}`)
      .join('\n')
    transforms =
      this.transforms.length != 0
        ? `    transforms: {\n${transforms}\n    }\n`
        : ''

    const footer = '  )'

    return `${header}${name}${namespace}${url}${headers}${introspectionHeaders}${transforms}${footer}`
  }
}
