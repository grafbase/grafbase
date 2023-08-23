import { Header, Headers, HeaderGenerator } from './header'
import {
  OpenApiQueryNamingStrategy,
  OpenApiTransform,
  OpenApiTransforms,
  OpenApiTransformsGenerator
} from './openapi/transforms'

/**
 * @deprecated Use the function form of transforms instead
 */
export interface OpenApiTransformParams {
  queryNaming: OpenApiQueryNamingStrategy
}

export interface OpenAPIParams {
  schema: string
  url?: string
  transforms?: OpenApiTransformParams | OpenApiTransformsGenerator
  headers?: HeaderGenerator
}

export class PartialOpenAPI {
  private name: string
  private schema: string
  private apiUrl?: string
  private transforms: OpenApiTransform[]
  private headers: Header[]
  private introspectionHeaders: Header[]

  constructor(name: string, params: OpenAPIParams) {
    const headers = new Headers()

    if (params.headers) {
      params.headers(headers)
    }

    const transforms = new OpenApiTransforms()
    if (typeof params.transforms === 'function') {
      params.transforms(transforms)
    } else if (params.transforms) {
      transforms.queryNaming(params.transforms.queryNaming)
    }

    this.name = name
    this.schema = params.schema
    this.apiUrl = params.url
    this.transforms = transforms.transforms
    this.headers = headers.headers
    this.introspectionHeaders = headers.introspectionHeaders
  }

  finalize(namespace?: boolean): OpenAPI {
    return new OpenAPI(
      this.name,
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
  private name: string
  private namespace?: boolean
  private schema: string
  private apiUrl?: string
  private transforms: OpenApiTransform[]
  private headers: Header[]
  private introspectionHeaders: Header[]

  constructor(
    name: string,
    schema: string,
    headers: Header[],
    introspectionHeaders: Header[],
    transforms: OpenApiTransform[],
    url?: string,
    namespace?: boolean
  ) {
    this.name = name
    this.namespace = namespace
    this.schema = schema
    this.apiUrl = url
    this.transforms = transforms
    this.headers = headers
    this.introspectionHeaders = introspectionHeaders
  }

  public toString(): string {
    const header = '  @openapi(\n'
    const name = `    name: "${this.name}"\n`

    let namespace;
    if (this.namespace === undefined || this.namespace === true)  {
      namespace = `    namespace: true\n`
    } else {
      namespace = ''
    }

    const url = this.apiUrl ? `    url: "${this.apiUrl}"\n` : ''
    const schema = `    schema: "${this.schema}"\n`

    let transforms = this.transforms
      .map((transform) => `      ${transform}`)
      .join('\n')
    transforms =
      this.transforms.length != 0
        ? `    transforms: {\n${transforms}\n    }\n`
        : ''

    let headers = this.headers.map((header) => `      ${header}`).join('\n')
    headers = headers ? `    headers: [\n${headers}\n    ]\n` : ''

    let introspectionHeaders = this.introspectionHeaders
      .map((header) => `      ${header}`)
      .join('\n')

    introspectionHeaders = introspectionHeaders
      ? `    introspectionHeaders: [\n${introspectionHeaders}\n    ]\n`
      : ''

    const footer = '  )'

    return `${header}${name}${namespace}${url}${schema}${transforms}${headers}${introspectionHeaders}${footer}`
  }
}
