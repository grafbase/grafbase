export interface NeonParams {
  url: string
}

export class PartialNeonAPI {
  private name: string
  private url: string

  constructor(name: string, params: NeonParams) {
    this.name = name
    this.url = params.url
  }

  finalize(namespace?: boolean): NeonAPI {
    return new NeonAPI(this.name, this.url, namespace)
  }
}

export class NeonAPI {
  private name: string
  private url: string
  private namespace?: boolean

  constructor(name: string, url: string, namespace?: boolean) {
    this.name = name
    this.url = url
    this.namespace = namespace
  }

  public toString(): string {
    const header = '  @neon(\n'
    const name = `    name: "${this.name}"\n`
    const url = `    url: "${this.url}"\n`

    let namespace
    if (this.namespace === undefined || this.namespace === true) {
      namespace = `    namespace: true\n`
    } else {
      namespace = '    namespace: false\n'
    }

    const footer = '  )'

    return `${header}${name}${url}${namespace}${footer}`
  }
}
