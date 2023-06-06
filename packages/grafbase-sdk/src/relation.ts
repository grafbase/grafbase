import { AuthRuleF } from './auth'
import { RelationListDefinition } from './typedefs/list'
import { Model } from './model'
import { AuthDefinition } from './typedefs/auth'

/**
 * A reference in a relation field. Can be a model, or a closure resolving to
 * a model.
 */
export type RelationRef = RelationF | Model

/**
 * A closure to define the referenced model in a relation. Useful if the model
 * is not defined yet. E.g. for self-relations or for models defined later in the
 * configuration.
 */
type RelationF = () => Model

/**
 * Defines relation field in a model.
 */
export class RelationDefinition {
  // For ambivalent relations, a name makes them distinct.
  // Rendered as `@relation(name: "relationName")`.
  private _relationName?: string
  // The model we refer from this field.
  private _referencedModel: RelationRef
  // True, if the field can hold a null value.
  private _isOptional: boolean

  /** @param {RelationRef} referencedModel */
  constructor(referencedModel: RelationRef) {
    this._referencedModel = referencedModel
    this._isOptional = false
  }

  /** Make the field nullable. */
  public optional(): RelationDefinition {
    this._isOptional = true

    return this
  }

  /** The field can hold multiple values */
  public list(): RelationListDefinition {
    return new RelationListDefinition(this)
  }

  /**
   * For ambivalent relations, a name makes them distinct.
   *
   * @param name - The name of the relation.
   */
  public name(name: string): RelationDefinition {
    this._relationName = name

    return this
  }

  /**
   * Set the field-level auth directive.
   *
   * @param rules - A closure to build the authentication rules.
   */
  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  /**
   * Gets the relations name
   */
  public get relationName(): string | undefined {
    return this._relationName
  }

  /**
   * Gets the referenced model
   */
  public get referencedModel(): RelationRef {
    return this._referencedModel
  }

  public get isOptional(): boolean {
    return this._isOptional
  }

  public toString(): string {
    let modelName

    if (typeof this._referencedModel === 'function') {
      modelName = this._referencedModel().name
    } else {
      modelName = this._referencedModel.name
    }

    const required = this.isOptional ? '' : '!'
    const relationAttribute = this._relationName
      ? ` @relation(name: "${this._relationName}")`
      : ''

    return `${modelName}${required}${relationAttribute}`
  }
}
