export type RateLimitingAllowAny = '*'

export interface HeaderCondition {
  headers: HeaderConditionSpec[]
}

export interface HeaderConditionSpec {
  name: string
  value: string[] | RateLimitingAllowAny
}

export interface OperationCondition {
  operations: string[] | RateLimitingAllowAny
}

export interface IpCondition {
  ips: string[] | RateLimitingAllowAny
}

export interface JwtCondition {
  jwt_claims: JwtConditionSpec[]
}

export interface JwtConditionSpec {
  name: string
  value: never | RateLimitingAllowAny
}

export interface RateLimitingRule {
  name: string
  limit: number
  duration: number
  condition: RateLimitingRuleCondition
}

type RateLimitingRuleCondition =
  | HeaderCondition
  | OperationCondition
  | JwtCondition
  | IpCondition

export interface RateLimitingParams {
  rules: RateLimitingRule[]
}

export class RateLimiting {
  rules: RateLimitingRule[]

  constructor(rateLimitingParams: RateLimitingParams) {
    this.rules = rateLimitingParams.rules
  }

  public toString(): string {
    return `\nextend schema\n  @rateLimiting(rules: ${renderRules(this.rules)}\n  )\n`
  }
}

function renderRules(rules: RateLimitingRule[]): string {
  const renderedRules = rules.map((rule) => {
    const name = `\n      name: "${rule.name}"`
    const limit = `,\n      limit: ${rule.limit}`
    const duration = `,\n      duration: ${rule.duration}`

    const condition = `,\n      condition:${renderCondition(rule.condition)}`

    return `{${name}${limit}${duration}${condition}\n    }`
  })

  return `[${renderedRules}]`
}

function renderCondition(condition: RateLimitingRuleCondition): string {
  if (isHeaderCondition(condition)) {
    return renderHeaderCondition(condition)
  }
  if (isJwtCondition(condition)) {
    return renderJwtCondition(condition)
  }
  if (isIpCondition(condition)) {
    return renderIpCondition(condition)
  }
  if (isOperationCondition(condition)) {
    return renderOperationCondition(condition)
  }
  return ''
}

function isHeaderCondition(
  condition: RateLimitingRuleCondition
): condition is HeaderCondition {
  return (condition as HeaderCondition).headers !== undefined
}

function renderHeaderCondition(condition: HeaderCondition): string {
  const headerConditions = condition.headers
    .map((headerCondition) => {
      let value = '"*"'
      if (headerCondition.value !== '*') {
        value = `[${headerCondition.value.map((value) => `"${value}"`).join(', ')}]`
      }

      return `{name: "${headerCondition.name}", value: ${value}}`
    })
    .join(',')

  return ` {\n        headers: [${headerConditions}]\n      }`
}

function isJwtCondition(
  condition: RateLimitingRuleCondition
): condition is JwtCondition {
  return (condition as JwtCondition).jwt_claims !== undefined
}

function renderJwtCondition(condition: JwtCondition): string {
  const conditions = condition.jwt_claims
    .map((condition) => {
      let value = '"*"'
      if (condition.value !== '*') {
        if (typeof condition.value === 'object') {
          value = `${JSON.stringify(JSON.stringify(condition.value))}`
        } else {
          value = `"${condition.value}"`
        }
      }

      return `{name: "${condition.name}", value: ${value}}`
    })
    .join(',')

  return ` {\n        jwt_claims: [${conditions}]\n      }`
}

function isIpCondition(
  condition: RateLimitingRuleCondition
): condition is IpCondition {
  return (condition as IpCondition).ips !== undefined
}

function renderIpCondition(condition: IpCondition): string {
  let rendered = '"*"'
  if (condition.ips !== '*') {
    rendered = `[${condition.ips.map((ip) => `"${ip}"`).join(',')}]`
  }

  return ` {\n        ips: ${rendered}\n      }`
}

function isOperationCondition(
  condition: RateLimitingRuleCondition
): condition is OperationCondition {
  return (condition as OperationCondition).operations !== undefined
}

function renderOperationCondition(condition: OperationCondition): string {
  let rendered = '"*"'
  if (condition.operations !== '*') {
    rendered = `[${condition.operations.map((operation) => `"${operation}"`).join(',')}]`
  }

  return ` {\n        operations: ${rendered}\n      }`
}
