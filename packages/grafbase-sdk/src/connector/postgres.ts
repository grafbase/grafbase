export interface PostgresParams {
  url: string
}

export class PartialPostgresAPI {
  private name: string
  private url: string

  constructor(name: string, params: PostgresParams) {
    this.name = name
    this.url = params.url
  }

  finalize(namespace?: boolean): PostgresAPI {
    return new PostgresAPI(this.name, this.url, namespace)
  }
}

export class PostgresAPI {
  private name: string
  private url: string
  private namespace?: boolean

  constructor(name: string, url: string, namespace?: boolean) {
    this.name = name
    this.url = url
    this.namespace = namespace
  }

  public toString(): string {
    const header = '  @postgres(\n'
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
