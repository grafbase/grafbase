import { AuthRuleF, AuthRules } from '../auth'
import { ReferenceDefinition } from '../reference'
import { RelationDefinition } from '../relation'
import { DefaultDefinition } from './default'
import { LengthLimitedStringDefinition } from './length-limited-string'
import { ScalarDefinition } from './scalar'
import { SearchDefinition } from './search'
import { UniqueDefinition } from './unique'

export type Authenticable =
  | ScalarDefinition
  | UniqueDefinition
  | DefaultDefinition
  | SearchDefinition
  | ReferenceDefinition
  | LengthLimitedStringDefinition
  | RelationDefinition

export class AuthDefinition {
  field: Authenticable
  authRules: AuthRules

  constructor(field: Authenticable, rules: AuthRuleF) {
    const authRules = new AuthRules()
    rules(authRules)

    this.authRules = authRules
    this.field = field
  }

  public toString(): string {
    // In field definition, concatenate all rules into one row.
    const rules = this.authRules.toString().replace(/\s\s+/g, ' ')

    return `${this.field} @auth(rules: ${rules})`
  }
}
