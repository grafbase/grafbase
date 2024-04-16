export type HttpMethod =
  | 'GET'
  | 'POST'
  | 'PUT'
  | 'DELETE'
  | 'HEAD'
  | 'OPTIONS'
  | 'CONNECT'
  | 'PATCH'
  | 'TRACE'

export type CorsAllowAny = '*'

/**
 * Cross-Origin Resource Sharing settings.
 */
export interface CorsParams {
  maxAge?: number
  allowedHeaders?: CorsAllowAny | string[]
  allowedMethods?: CorsAllowAny | HttpMethod[]
  exposedHeaders?: CorsAllowAny | string[]
  allowCredentials?: boolean
  allowedOrigins?: CorsAllowAny | URL[]
}

export class Cors {
  private params: CorsParams

  constructor(params: CorsParams) {
    this.params = params
  }

  public toString(): string {
    const allowCredentials = this.params.allowCredentials
      ? `\n    allowCredentials: ${this.params.allowCredentials}`
      : '\n    allowCredentials: false'

    const allowedHeaders = this.params.allowedHeaders
      ? `,\n    allowedHeaders: ${renderAnyOrStrings(this.params.allowedHeaders)}`
      : ''

    const allowedMethods = this.params.allowedMethods
      ? `,\n    allowedMethods: ${renderAnyOrStrings(this.params.allowedMethods)}`
      : ''

    const exposedHeaders = this.params.exposedHeaders
      ? `,\n    exposedHeaders: ${renderAnyOrStrings(this.params.exposedHeaders)}`
      : ''

    const allowedOrigins = this.params.allowedOrigins
      ? `,\n    allowedOrigins: ${renderAllowedOrigins(this.params.allowedOrigins)}`
      : ''

    const maxAge = this.params.maxAge
      ? `,\n    maxAge: ${this.params.maxAge}`
      : ''

    return `extend schema\n  @cors(${allowCredentials}${maxAge}${allowedHeaders}${allowedMethods}${exposedHeaders}${allowedOrigins}\n  )\n\n`
  }
}

function renderAllowedOrigins(origins: CorsAllowAny | URL[]): string {
  if (origins === '*') {
    return '"*"'
  } else {
    return `[${origins.map((origin) => `"${origin}"`).join(', ')}]`
  }
}

function renderAnyOrStrings(headers: CorsAllowAny | string[]): string {
  if (headers === '*') {
    return '"*"'
  } else {
    return `[${headers.map((header) => `"${header}"`).join(', ')}]`
  }
}
