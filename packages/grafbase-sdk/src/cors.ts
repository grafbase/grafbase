export type CorsAllowAny = '*'

/**
 * Cross-Origin Resource Sharing settings.
 */
export interface CorsParams {
  maxAge?: number
  allowedOrigins?: CorsAllowAny | URL[]
}

export class Cors {
  private params: CorsParams

  constructor(params: CorsParams) {
    this.params = params
  }

  public toString(): string {
    const allowedOrigins = `\n    allowedOrigins: ${renderAllowedOrigins(
      this.params.allowedOrigins
    )}`

    const maxAge = this.params.maxAge
      ? `,\n    maxAge: ${this.params.maxAge}`
      : ''

    return `extend schema\n  @cors(${allowedOrigins}${maxAge}\n  )\n\n`
  }
}

function renderAllowedOrigins(origins?: CorsAllowAny | URL[]): string {
  if (origins === '*') {
    return '"*"'
  } else {
    return origins
      ? `[${origins.map((origin) => `"${origin}"`).join(', ')}]`
      : '[]'
  }
}
