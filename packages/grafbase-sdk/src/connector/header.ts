export type HeaderGenerator = (headers: Headers) => any

/**
 * Header used in connector calls.
 */
export class Header {
  private name: string
  private value: string

  constructor(name: string, value: string) {
    this.name = name
    this.value = value
  }

  public toString(): string {
    return `{ name: "${this.name}", value: "${this.value}" }`
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
   * @param name - The name of the header
   * @param value - The value of the header
   */
  public static(name: string, value: string) {
    this.headers.push(new Header(name, value))
  }

  /**
   * Creates a header used in introspection requests.
   *
   * @param name - The name of the header
   * @param value - The value of the header
   */
  public introspection(name: string, value: string) {
    this.introspectionHeaders.push(new Header(name, value))
  }
}
