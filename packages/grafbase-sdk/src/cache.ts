/**
 * Defines a cached type with fields.
 */
export interface StructuredCacheRuleType {
  name: string
  fields?: string[]
}

/**
 * Defines a type to be cached. Can be a single type, multiple types
 * or more granularly types with specific fields.
 */
export type CachedType = string | string[] | StructuredCacheRuleType[]

/**
 * Defines a single global cache rule.
 */
export interface CacheRuleParam {
  types: CachedType
  maxAge: number
  staleWhileRevalidate: number
}

/**
 * Defines global cache rules.
 */
export interface CacheParams {
  rules: CacheRuleParam[]
}

export class GlobalCache {
  params: CacheParams

  constructor(params: CacheParams) {
    this.params = params
  }

  public toString(): string {
    const rules = this.params.rules.map((rule) => {
      var types
      if (typeof rule.types === 'string') {
        types = `"${rule.types}"`
      } else {
        const inner = rule.types
          .map((type) => {
            if (typeof type === 'string') {
              return `"${type}"`
            } else {
              var fields = type.fields ? type.fields.map((field) => `"${field}"`).join(',') : ''
              fields = fields ? `,\n        fields: [${fields}]\n` : '\n'

              return `{\n        name: "${type.name}"${fields}      }`
            }
          })
          .join(', ')

        types = `[${inner}]`
      }

      return `    {\n      types: ${types},\n      maxAge: ${rule.maxAge},\n      staleWhileRevalidate: ${rule.staleWhileRevalidate}\n    }`
    })

    return `extend schema\n  @cache(rules: [\n${rules}\n  ])\n\n`
  }
}
