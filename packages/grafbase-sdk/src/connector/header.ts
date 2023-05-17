export type HeaderGenerator = (headers: Headers) => any
export type PartialHeaderGenerator = (headers: PartialHeaders) => any /**

/**
* Header used in connector calls.
*/
export class Header {
  name: string
  value: string

  constructor(name: string, value: string) {
    this.name = name
    this.value = value
  }

  public toString(): string {
    return `{ name: "${this.name}", value: "${this.value}" }`
  }
}

/**
 * An accumulator class to gather headers for a connector.
 */
export class PartialHeaders {
  headers: Header[]

  constructor() {
    this.headers = []
  }

  /**
   * Creates a header used in client and introspection requests.
   *
   * @param name - The name of the header
   * @param value - The value of the header
   */
  public static(name: string, value: string) {
    this.headers.push(new Header(name, value))
  }
}

/**
 * An accumulator class to gather headers for a connector which supports
 * introspection headers.
 */
export class Headers extends PartialHeaders {
  introspectionHeaders: Header[]

  constructor() {
    super()
    this.introspectionHeaders = []
  }

  /**
   * Creates a header used only in introspection requests.
   *
   * @param name - The name of the header
   * @param value - The value of the header
   */
  public introspection(name: string, value: string) {
    this.introspectionHeaders.push(new Header(name, value))
  }
}
