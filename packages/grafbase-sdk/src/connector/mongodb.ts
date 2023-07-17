import { ModelFields, MongoDBModel } from './mongodb/model'

export interface MongoDBParams {
  name: string
  apiKey: string
  appId: string
  dataSource: string
  database: string
}

export class PartialMongoDBAPI {
  private name: string
  private apiKey: string
  private appId: string
  private dataSource: string
  private database: string
  private models: MongoDBModel[]

  constructor(params: MongoDBParams) {
    this.name = params.name
    this.apiKey = params.apiKey
    this.appId = params.appId
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

  finalize(namespace?: string): MongoDBAPI {
    return new MongoDBAPI(
      this.name,
      this.apiKey,
      this.appId,
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
  private appId: string
  private dataSource: string
  private database: string
  private namespace?: string
  public models: MongoDBModel[]

  constructor(
    name: string,
    apiKey: string,
    appId: string,
    dataSource: string,
    database: string,
    models: MongoDBModel[],
    namespace?: string
  ) {
    this.name = name
    this.apiKey = apiKey
    this.appId = appId
    this.dataSource = dataSource
    this.database = database
    this.namespace = namespace
    this.models = models
  }

  public toString(): string {
    const header = '  @mongodb(\n'
    const name = `    name: "${this.name}"\n`
    const apiKey = `    apiKey: "${this.apiKey}"\n`
    const appId = `    appId: "${this.appId}"\n`
    const dataSource = `    dataSource: "${this.dataSource}"\n`
    const database = `    database: "${this.database}"\n`

    const namespace = this.namespace
      ? `    namespace: "${this.namespace}"\n`
      : ''

    const footer = '  )'

    return `${header}${namespace}${name}${apiKey}${appId}${dataSource}${database}${footer}`
  }
}
