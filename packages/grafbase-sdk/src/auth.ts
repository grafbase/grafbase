import { JWKSAuth } from './auth/jwks'
import { JWTAuth } from './auth/jwt'
import { OpenIDAuth } from './auth/openid'

export type AuthProvider = OpenIDAuth | JWTAuth | JWKSAuth
export type AuthRuleF = (rules: AuthRules) => any
export type AuthOperation =
  | 'get'
  | 'list'
  | 'read'
  | 'create'
  | 'update'
  | 'delete'

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

export type AuthStrategy = 'private' | 'owner' | AuthGroups

export class AuthRule {
  strategy: AuthStrategy
  operations: AuthOperation[]

  constructor(strategy: AuthStrategy) {
    this.strategy = strategy
    this.operations = []
  }

  public get(): AuthRule {
    return this.operation('get')
  }

  public list(): AuthRule {
    return this.operation('list')
  }

  public read(): AuthRule {
    return this.operation('read')
  }

  public create(): AuthRule {
    return this.operation('create')
  }

  public update(): AuthRule {
    return this.operation('update')
  }

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

export class AuthRules {
  rules: AuthRule[]

  constructor() {
    this.rules = []
  }

  public private(): AuthRule {
    const rule = new AuthRule('private')

    this.rules.push(rule)

    return rule
  }

  public owner(): AuthRule {
    const rule = new AuthRule('owner')

    this.rules.push(rule)

    return rule
  }

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
  providers: AuthProvider[]
  rules: AuthRuleF
}

export class Authentication {
  providers: AuthProvider[]
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
