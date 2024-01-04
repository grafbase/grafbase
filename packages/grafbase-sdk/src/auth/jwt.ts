/**
 * Input parameters to define a JWT auth provider. Requires `issuer` and `secret`
 * to be defined, and optionally supports the `clientId` and `groupsClaim`
 * definitions.
 *
 * `clientId` should be defined for providers that sign tokens with the same
 * `iss` value. The value of `clientId` is checked against the `aud` claim
 * inside the JWT.
 *
 * `groupsClaim` should be defined for group-based auth to use a custom claim
 * path.
 */
export interface JWTParams {
  issuer: string
  secret: string
  clientId?: string
  groupsClaim?: string
}

/**
 * Grafbase supports a symmetric JWT provider that you can use to authorize
 * requests using a JWT signed by yourself or a third-party service.
 */
export class JWTAuth {
  private issuer: string
  private secret: string
  private clientId?: string
  private groupsClaim?: string

  constructor(params: JWTParams) {
    this.issuer = params.issuer
    this.secret = params.secret
    this.clientId = params.clientId
    this.groupsClaim = params.groupsClaim
  }

  public toString(): string {
    const clientId = this.clientId ? `, clientId: "${this.clientId}"` : ''
    const groupsClaim = this.groupsClaim
      ? `, groupsClaim: "${this.groupsClaim}"`
      : ''

    return `{ type: jwt, issuer: "${this.issuer}", secret: "${this.secret}"${clientId}${groupsClaim} }`
  }
}

/**
 * Input parameters to define a JWT auth provider for a Federated Graph.
 *
 * JWT are validated following the RFC7519 specification. The only difference is that the `exp` claim is required.
 *
 * JWT tokens will have their signature validated with the keys found at the URL `jwks.url`.
 * They're expected to follow the "JSON Web Key (JWK)" (RFC 7517) format. The JWKS will be
 * regularly updated by `pollInterval` which is `'60s'` by default and cannot be lower.
 *
 * If provided:
 * - `jwks.issuer` must match the `iss` claim.
 * - `jwks.audience` match match the `aud` claim.
 *
 * It is very strongly recommended to specify the `jwks.audience`. You're most likely accepting JWTs
 * which weren't issued for your API otherwise.
 *
 * `name` is used for logging & error messages.
 *
 * `header.name` and `header.valuePrefix` can be used to customize where the JWT token is in the HTTP headers. They default
 * to `'Authorization'` and `'Bearer '` respectively.
 */
export interface JWTParamsV2 {
  name?: string
  jwks: {
    url: string
    issuer?: string
    audience?: string
    pollInterval?: string
  }
  header?: {
    name?: string
    valuePrefix?: string
  }
}

/**
 * JWT authentication for Federated Graphs.
 */
export class JWTAuthV2 {
  private params: JWTParamsV2

  constructor(params: JWTParamsV2) {
    this.params = params
  }

  public toString(): string {
    const name = this.params.name ? `, name: "${this.params.name}"` : ''
    const issuer = this.params.jwks.issuer
      ? `, issuer: "${this.params.jwks.issuer}"`
      : ''
    const audience = this.params.jwks.audience
      ? `, audience: "${this.params.jwks.audience}"`
      : ''
    const pollInterval = this.params.jwks.pollInterval
      ? `, pollInterval: "${this.params.jwks.pollInterval}"`
      : ''

    let header = ''
    if (this.params.header?.name || this.params.header?.valuePrefix) {
      const headerName = this.params.header?.name
        ? `name: "${this.params.header.name}"`
        : 'name: "Authorization"'
      const headerValuePrefix = this.params.header?.valuePrefix
        ? `, valuePrefix: "${this.params.header.valuePrefix}"`
        : ''
      header = `, header: { ${headerName}${headerValuePrefix} }`
    }
    return `{ type: jwt, jwks: { url: "${this.params.jwks.url}"${issuer}${audience}${pollInterval} }${header}${name} }`
  }
}
