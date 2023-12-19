import { FixedLengthArray } from 'type-fest'
import { JWKSAuth } from './auth/jwks'
import { JWTAuth } from './auth/jwt'
import { OpenIDAuth } from './auth/openid'
import { Authorizer } from './auth/authorizer'

/**
 * A list of authentication providers which can be used in the configuration.
 */
export type AuthProvider = OpenIDAuth | JWTAuth | JWKSAuth | Authorizer

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
  | 'introspection'

/**
 * A list of supported authentication strategies.
 */
export type AuthStrategy = 'public' | 'private' | AuthGroups

/**
 * A builder to greate auth groups.
 */
export class AuthGroups {
  private groups: string[]

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
  private strategy: AuthStrategy
  private operations: AuthOperation[]

  constructor(strategy: AuthStrategy) {
    this.strategy = strategy
    this.operations = []
  }

  /** Allow the `get` operation for the given strategy. */
  public get(): AuthRule {
    return this.operation('get')
  }

  /** Allow the `list` operation for the given strategy. */
  public list(): AuthRule {
    return this.operation('list')
  }

  /** Allow the `read` operation for the given strategy. */
  public read(): AuthRule {
    return this.operation('read')
  }

  /** Allow the `create` operation for the given strategy. */
  public create(): AuthRule {
    return this.operation('create')
  }

  /** Allow the `update` operation for the given strategy. */
  public update(): AuthRule {
    return this.operation('update')
  }

  /** Allow the `delete` operation for the given strategy. */
  public delete(): AuthRule {
    return this.operation('delete')
  }

  /** Allow the `introspection` operation for the given strategy. */
  public introspection(): AuthRule {
    return this.operation('introspection')
  }

  public toString(): string {
    const allow = `allow: ${this.strategy}`

    let ops = this.operations.map((op) => `${op}`).join(', ')
    ops = ops ? `, operations: [${ops}]` : ''

    return `{ ${allow}${ops} }`
  }

  private operation(op: AuthOperation): AuthRule {
    this.operations.push(op)

    return this
  }
}

/**
 * A builder to generate a set of rules to the auth attribute.
 */
export class AuthRules {
  private rules: AuthRule[]

  constructor() {
    this.rules = []
  }

  /**
   * Allow public access.
   */
  public public(): AuthRule {
    const rule = new AuthRule('public')

    this.rules.push(rule)

    return rule
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
   * Allow access to users of a group.
   *
   * @param groups - A list of groups with access.
   */
  public groups(groups: string[]): AuthRule {
    const rule = new AuthRule(new AuthGroups(groups))

    this.rules.push(rule)

    return rule
  }

  public toString(): string {
    let rules = this.rules.map((rule) => `      ${rule}`).join('\n')

    if (rules) {
      rules = `[\n${rules}\n    ]`
    } else {
      rules = ''
    }

    return rules
  }
}

type RequireAtLeastOne<T> = {
  [K in keyof T]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<keyof T, K>>>
}[keyof T]

export type AuthParams = RequireAtLeastOne<{
  providers?: FixedLengthArray<AuthProvider, 1>
  rules?: AuthRuleF
}>

export class Authentication {
  private providers?: FixedLengthArray<AuthProvider, 1>
  private rules: AuthRules

  constructor(params: AuthParams) {
    this.providers = params.providers

    const rules = new AuthRules()
    params.rules?.(rules)

    this.rules = rules
  }

  public toString(): string {
    let providers = this.providers
      ? this.providers.map(String).join('\n      ')
      : ''

    if (providers) {
      providers = `\n    providers: [\n      ${providers}\n    ]`
    }

    let rules = this.rules.toString()

    if (rules) {
      rules = `\n    rules: ${rules}`
    }

    return `extend schema\n  @auth(${providers}${rules}\n  )`
  }
}
