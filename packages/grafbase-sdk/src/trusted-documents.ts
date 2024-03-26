/**
 * Configures [Trusted Documents](https://grafbase.com/docs/security/trusted-documents).
 */
export interface TrustedDocumentsParams {
  /**
   * Enforce the use of trusted documents.
   */
  enabled: boolean
  /*
   * A header that can be used to send arbitrary queries.
   */
  bypassHeader?: {
    name: string
    value: string
  }
}

export class TrustedDocuments {
  private params: TrustedDocumentsParams

  constructor(params: TrustedDocumentsParams) {
    this.params = params
  }
  
  public toString(): string {
    if (!this.params.enabled) {
      return ''
    }

    const args = this.params.bypassHeader
      ? `(bypassHeaderName: ${JSON.stringify(this.params.bypassHeader.name)}, byPassHeaderValue: ${JSON.stringify(this.params.bypassHeader.value)})`
      : ''

    return `extend schema\n  @trustedDocuments${args}`
  }
}
