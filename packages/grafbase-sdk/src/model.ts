import { AuthRuleF, AuthRules } from './auth'
import { Field } from './field'
import { TypeCacheParams, TypeLevelCache } from './typedefs/cache'
import { validateIdentifier } from './validation'

export class Model {
  private _name: string
  protected fields: Field[]
  protected authRules?: AuthRules
  protected cacheDirective?: TypeLevelCache

  constructor(name: string) {
    validateIdentifier(name)

    this._name = name
    this.fields = []
  }

  /**
   * Get the name of the model.
   */
  public get name(): string {
    return this._name
  }

  /**
   * Set the per-model `@auth` directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): this {
    const authRules = new AuthRules()
    rules(authRules)
    this.authRules = authRules

    return this
  }

  /**
   * Set the per-model `@cache` directive.
   *
   * @param params - The cache definition parameters.
   */
  public cache(params: TypeCacheParams): this {
    this.cacheDirective = new TypeLevelCache(params)

    return this
  }
}
