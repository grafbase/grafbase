export type HeaderGenerator = (headers: Headers) => any
export type HeaderValue =
  | { type: 'static'; value: string }
  | { type: 'forward'; from: string }

/**
 * Header used in connector calls.
 */
export class Header {
  private name: string
  private value: HeaderValue

  constructor(name: string, value: HeaderValue) {
    this.name = name
    this.value = value
  }

  public toString(): string {
    const valueStr =
      this.value.type === 'static'
        ? `value: "${this.value.value}"`
        : `forward: "${this.value.from}"`

    return `{ name: "${this.name}", ${valueStr} }`
  }
}

/**
 * An accumulator class to gather headers for a connector which supports
 * introspection headers.
 */
export class Headers {
  private _headers: Header[]
  private _introspectionHeaders: Header[]

  constructor() {
    this._headers = []
    this._introspectionHeaders = []
  }

  /**
   * All headers used in client requests.
   */
  public get headers(): Header[] {
    return this._headers
  }

  /**
   * All headers used in introspection requests.
   */
  public get introspectionHeaders(): Header[] {
    return this._introspectionHeaders
  }

  /**
   * Creates a header used in client requests.
   *
   * @deprecated Use set instead
   * @param name - The name of the header
   * @param value - The value of the header
   */
  public static(name: string, value: string) {
    this.headers.push(new Header(name, { type: 'static', value }))
  }

  /**
   * Creates a header used in client requests.
   *
   * @param name - The name of the header
   * @param value - The value for the header.  Either a hardcoded string or a header name to forward from.
   */
  public set(name: string, value: string | { forward: string }) {
    if (typeof value === 'string') {
      this.headers.push(new Header(name, { type: 'static', value }))
    } else {
      this.headers.push(
        new Header(name, { type: 'forward', from: value.forward })
      )
    }
  }

  /**
   * Creates a header used in introspection requests.
   *
   * @param name - The name of the header
   * @param value - The value of the header
   */
  public introspection(name: string, value: string) {
    this.introspectionHeaders.push(new Header(name, { type: 'static', value }))
  }
}
