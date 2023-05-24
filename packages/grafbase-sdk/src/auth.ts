import { FixedLengthArray } from 'type-fest'
import { JWKSAuth } from './auth/jwks'
import { JWTAuth } from './auth/jwt'
import { OpenIDAuth } from './auth/openid'

/**
* A list of authentication providers which can be used in the configuration.
*/
export type AuthProvider = OpenIDAuth | JWTAuth | JWKSAuth

/**
* A closure to define authentication rules.
*/
export type AuthRuleF = (rules: AuthRules) => any

/**
* A list of supported authenticated operations.
*/
export type AuthOperation =
  | 'get'
  | 'list'
  | 'read'
  | 'create'
  | 'update'
  | 'delete'

/**
* A list of supported authentication strategies.
*/
export type AuthStrategy = 'private' | 'owner' | AuthGroups

/**
* A builder to greate auth groups.
*/
export class AuthGroups {
  groups: string[]

  constructor(groups: string[]) {
    this.groups = groups
  }

  public toString(): string {
    const groups = this.groups.map((g) => `"${g}"`).join(', ')
    return `groups, groups: [${groups}]`
  }
}

/**
* A builder to create a rule to the auth attribute.
*/
export class AuthRule {
  strategy: AuthStrategy
  operations: AuthOperation[]

  constructor(strategy: AuthStrategy) {
    this.strategy = strategy
    this.operations = []
  }

  /** Allows the `get` operation for the given strategy. */
  public get(): AuthRule {
    return this.operation('get')
  }

  /** Allows the `list` operation for the given strategy. */
  public list(): AuthRule {
    return this.operation('list')
  }

  /** Allows the `read` operation for the given strategy. */
  public read(): AuthRule {
    return this.operation('read')
  }

  /** Allows the `create` operation for the given strategy. */
  public create(): AuthRule {
    return this.operation('create')
  }

  /** Allows the `update` operation for the given strategy. */
  public update(): AuthRule {
    return this.operation('update')
  }

  /** Allows the `delete` operation for the given strategy. */
  public delete(): AuthRule {
    return this.operation('delete')
  }

  public toString(): string {
    const allow = `allow: ${this.strategy}`

    var ops = this.operations.map((op) => `${op}`).join(', ')
    ops = ops ? `, operations: [${ops}]` : ''

    return `{ ${allow}${ops} }`
  }

  operation(op: AuthOperation): AuthRule {
    this.operations.push(op)

    return this
  }
}

/**
* A builder to generate a set of rules to the auth attribute.
*/
export class AuthRules {
  rules: AuthRule[]

  constructor() {
    this.rules = []
  }

  /**
  * Allow access to any signed-in user.
  */
  public private(): AuthRule {
    const rule = new AuthRule('private')

    this.rules.push(rule)

    return rule
  }

  /**
  * Allow access to the owner only.
  */
  public owner(): AuthRule {
    const rule = new AuthRule('owner')

    this.rules.push(rule)

    return rule
  }

  /**
  * Allow access to users of a group.
  */
  public groups(groups: string[]): AuthRule {
    const rule = new AuthRule(new AuthGroups(groups))

    this.rules.push(rule)

    return rule
  }

  public toString(): string {
    var rules = this.rules.map((rule) => `      ${rule}`).join('\n')

    if (rules) {
      rules = `[\n${rules}\n    ]`
    } else {
      rules = ''
    }

    return rules
  }
}

export interface AuthParams {
  providers: FixedLengthArray<AuthProvider, 1>
  rules: AuthRuleF
}

export class Authentication {
  providers: FixedLengthArray<AuthProvider, 1>
  rules: AuthRules

  constructor(params: AuthParams) {
    this.providers = params.providers

    const rules = new AuthRules()
    params.rules(rules)

    this.rules = rules
  }

  public toString(): string {
    const providers = this.providers.map(String).join('\n      ')
    var rules = this.rules.toString()

    if (rules) {
      rules = `\n    rules: ${rules}`
    } else {
      rules = ''
    }

    return `extend schema\n  @auth(\n    providers: [\n      ${providers}\n    ]${rules}\n  )`
  }
}
