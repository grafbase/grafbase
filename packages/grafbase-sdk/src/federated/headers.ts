import { Header } from '../connector/header'

/**
 * An accumulator class to gather headers for a federated graph
 */
export class FederatedGraphHeaders {
  private defaultHeaders: Header[]
  private subgraphs: { [name: string]: Header[] }

  constructor() {
    this.defaultHeaders = []
    this.subgraphs = {}
  }

  /**
   * Sets a default header to be sent to all subgraphs
   *
   * @param name - The name of the header
   * @param value - The value for the header.  Either a hardcoded string or a header name to forward from.
   */
  public set(name: string, value: string | { forward: string }) {
    if (typeof value === 'string') {
      this.defaultHeaders.push(new Header(name, { type: 'static', value }))
    } else {
      this.defaultHeaders.push(
        new Header(name, { type: 'forward', from: value.forward })
      )
    }
  }

  /**
   * Returns a builder for setting a specific subgraphs headers
   *
   * @param name - The name of the subgraph
   */
  public subgraph(name: string): FederatedSubgraphHeaders {
    this.subgraphs[name] ||= []
    return new FederatedSubgraphHeaders(this.subgraphs[name])
  }

  public toString(): string {
    const defaultHeaders =
      this.defaultHeaders.length !== 0
        ? `\n  @allSubgraphs(headers: [${this.defaultHeaders
            .map(String)
            .join(', ')}])`
        : ''

    const subgraphs =
      Object.keys(this.subgraphs).length !== 0
        ? Object.entries(this.subgraphs).map(
            ([name, headers]) =>
              `\n  @subgraph(name: "${name}", headers: [${headers
                .map(String)
                .join(', ')}])`
          )
        : ''

    return `${defaultHeaders}${subgraphs}`
  }
}

export class FederatedSubgraphHeaders {
  private headers: Header[]

  constructor(headers: Header[]) {
    this.headers = headers
  }

  /**
   * Sets a header to be sent to this subgraph
   *
   * @param name - The name of the header
   * @param value - The value for the header.  Either a hardcoded string or a header name to forward from.
   */
  public set(
    name: string,
    value: string | { forward: string }
  ): FederatedSubgraphHeaders {
    if (typeof value === 'string') {
      this.headers.push(new Header(name, { type: 'static', value }))
    } else {
      this.headers.push(
        new Header(name, { type: 'forward', from: value.forward })
      )
    }

    return this
  }
}
