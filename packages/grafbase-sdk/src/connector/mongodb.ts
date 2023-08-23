import { ModelFields, MongoDBModel } from './mongodb/model'

export interface MongoDBParams {
  url: string
  apiKey: string
  dataSource: string
  database: string
}

export class PartialMongoDBAPI {
  private name: string
  private url: string
  private apiKey: string
  private dataSource: string
  private database: string
  private models: MongoDBModel[]

  constructor(name: string, params: MongoDBParams) {
    this.name = name
    this.url = params.url
    this.apiKey = params.apiKey
    this.dataSource = params.dataSource
    this.database = params.database
    this.models = []
  }

  /**
   * Creates a new model type with an access to this MongoDB data source.
   *
   * @param name - The name of the model
   * @param fields - The fields of the model
   */
  public model(name: string, fields: ModelFields): MongoDBModel {
    const model = Object.entries(fields).reduce(
      (model, [name, definition]) => model.field(name, definition),
      new MongoDBModel(name, this.name)
    )

    this.models.push(model)

    return model
  }

  finalize(namespace?: boolean): MongoDBAPI {
    return new MongoDBAPI(
      this.name,
      this.apiKey,
      this.url,
      this.dataSource,
      this.database,
      this.models,
      namespace
    )
  }
}

export class MongoDBAPI {
  private name: string
  private apiKey: string
  private url: string
  private dataSource: string
  private database: string
  private namespace?: boolean
  public models: MongoDBModel[]

  constructor(
    name: string,
    apiKey: string,
    url: string,
    dataSource: string,
    database: string,
    models: MongoDBModel[],
    namespace?: boolean
  ) {
    this.name = name
    this.apiKey = apiKey
    this.url = url
    this.dataSource = dataSource
    this.database = database
    this.namespace = namespace
    this.models = models
  }

  public toString(): string {
    const header = '  @mongodb(\n'
    const name = `    name: "${this.name}"\n`
    const url = `    url: "${this.url}"\n`
    const apiKey = `    apiKey: "${this.apiKey}"\n`
    const dataSource = `    dataSource: "${this.dataSource}"\n`
    const database = `    database: "${this.database}"\n`
    
    let namespace;
    if (this.namespace === undefined || this.namespace === true)  {
      namespace = `    namespace: true\n`
    } else {
      namespace = ''
    }

    const footer = '  )'

    return `${header}${namespace}${name}${url}${apiKey}${dataSource}${database}${footer}`
  }
}
