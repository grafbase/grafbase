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
export type CachedTypes = string | string[] | StructuredCacheRuleType[]

/**
 * Defines the invalidation strategy on mutations for the cache.
 * - 'entity' will invalidate all cache values that return an entity with an
 *   `id`.
 * - `type` will invalidate all cache values that have type in them
 * - `list` will invalidate all cache values that have lists of the type in them
 * - `{ field: string }` will invalidate all cache values that return an entity
 *   with the given field in them
 */
export type MutationInvalidation =
  | 'type'
  | 'entity'
  | 'list'
  | { field: string }

/**
 * Defines a single global cache rule.
 */
export interface CacheRuleParam {
  types: CachedTypes
  maxAge: number
  staleWhileRevalidate?: number
  mutationInvalidation?: MutationInvalidation
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
      const types = `\n      types: ${renderTypes(rule.types)}`
      const maxAge = `,\n      maxAge: ${rule.maxAge}`

      const staleWhileRevalidate = rule.staleWhileRevalidate
        ? `,\n      staleWhileRevalidate: ${rule.staleWhileRevalidate}`
        : ''

      const mutationInvalidation = rule.mutationInvalidation
        ? `,\n      mutationInvalidation: ${renderMutationInvalidation(
            rule.mutationInvalidation
          )}`
        : ''

      return `    {${types}${maxAge}${staleWhileRevalidate}${mutationInvalidation}\n    }`
    })

    return `extend schema\n  @cache(rules: [\n${rules}\n  ])\n\n`
  }
}

export function renderMutationInvalidation(val: MutationInvalidation): string {
  if (typeof val === 'object') {
    return `{ field: "${val.field}" }`
  } else {
    return val
  }
}

function renderTypes(types: CachedTypes): string {
  if (typeof types === 'string') {
    return `"${types}"`
  } else {
    const inner = types
      .map((type) => {
        if (typeof type === 'string') {
          return `"${type}"`
        } else {
          var fields = type.fields
            ? type.fields.map((field) => `"${field}"`).join(',')
            : ''
          fields = fields ? `,\n        fields: [${fields}]\n` : '\n'

          return `{\n        name: "${type.name}"${fields}      }`
        }
      })
      .join(', ')

    return `[${inner}]`
  }
}
