export enum SubscriptionTransport {
  GraphQlOverWebsockets
}

export interface SubscriptionTransportOptions {
  url: string
}

/**
 * An accumulator class to gather headers for a federated graph
 */
export class FederatedGraphSubscriptions {
  private subgraphs: { [name: string]: FederatedSubgraphSubscription }

  constructor() {
    this.subgraphs = {}
  }

  /**
   * Returns a builder for setting a specific subgraphs headers
   *
   * @param name - The name of the subgraph
   */
  public subgraph(name: string): FederatedSubgraphSubscription {
    this.subgraphs[name] ||= new FederatedSubgraphSubscription()

    return this.subgraphs[name]
  }

  public toString(): string {
    const subgraphs =
      Object.keys(this.subgraphs).length !== 0
        ? Object.entries(this.subgraphs).map(([name, settings]) =>
            settings.websocketUrl
              ? `\n  @subgraph(name: "${name}", websocketUrl: "${settings.websocketUrl}")`
              : ''
          )
        : ''

    return `${subgraphs}`
  }
}

export class FederatedSubgraphSubscription {
  private _websocketUrl: string | null

  constructor() {
    this._websocketUrl = null
  }

  get websocketUrl(): string | null {
    return this._websocketUrl
  }

  /**
   * Sets a header to be sent to this subgraph
   *
   * @param name - The name of the header
   * @param value - The value for the header.  Either a hardcoded string or a header name to forward from.
   */
  public transport(
    transport: SubscriptionTransport,
    options?: SubscriptionTransportOptions
  ): FederatedSubgraphSubscription {
    // Transport does nothing for now because we only support websockets.
    if (options?.url) {
      this._websocketUrl = options.url
    }

    return this
  }
}
