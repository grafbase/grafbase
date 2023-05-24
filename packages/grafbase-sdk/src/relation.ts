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
  relationName?: string
  // The model we refer from this field.
  referencedModel: RelationRef
  // True, if the field can hold a null value.
  isOptional: boolean

  /** @param {RelationRef} referencedModel */
  constructor(referencedModel: RelationRef) {
    this.referencedModel = referencedModel
    this.isOptional = false
  }

  /** Make the field nullable. */
  public optional(): RelationDefinition {
    this.isOptional = true

    return this
  }

  /** The field can hold multiple values */
  public list(): RelationListDefinition {
    return new RelationListDefinition(this)
  }

  /** For ambivalent relations, a name makes them distinct. */
  public name(name: string): RelationDefinition {
    this.relationName = name

    return this
  }

  /** Protect the field with authentication rules. */
  public auth(rules: AuthRuleF): AuthDefinition {
    return new AuthDefinition(this, rules)
  }

  public toString(): string {
    let modelName

    if (typeof this.referencedModel === 'function') {
      modelName = this.referencedModel().name
    } else {
      modelName = this.referencedModel.name
    }

    const required = this.isOptional ? '' : '!'
    const relationAttribute = this.relationName
      ? ` @relation(name: ${this.relationName})`
      : ''

    return `${modelName}${required}${relationAttribute}`
  }
}
