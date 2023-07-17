import { connect } from '@planetscale/database'
import { GraphQLError } from 'graphql'

import { config, options } from '../../lib'

const conn = connect(config)

export default async function ProductsUpdate(_: unknown, args: any) {
  const { by, input } = args

  let updateClauses: string[] = []
  let params: (string | number | boolean)[] = []
  let selectStatement: string = ''
  let selectParams: (string | number)[] = []

  Object.entries(input).forEach(([field, value]) => {
    if (
      value !== undefined &&
      value !== null &&
      !(typeof value === 'object' && Object.keys(value).length === 0)
    ) {
      if (
        typeof value === 'string' ||
        typeof value === 'number' ||
        typeof value === 'boolean'
      ) {
        updateClauses.push(`${field} = ?`)
        params.push(value)
      }
    }
  })

  if (params.length === 0) {
    throw new Error('At least one field to update must be provided.')
  }

  let updateStatement = 'UPDATE Products SET ' + updateClauses.join(', ')

  Object.entries(by).forEach(([field, value]) => {
    if (
      value !== undefined &&
      value !== null &&
      (typeof value === 'string' || typeof value === 'number')
    ) {
      updateStatement += ` WHERE ${field} = ?`
      params.push(value)
      selectStatement = `SELECT * FROM Products WHERE ${field} = ?`
      selectParams = [value]
    }
  })

  if (!selectStatement) {
    throw new GraphQLError('ID or Slug must be provided')
  }

  try {
    const [_, results] = await conn.transaction(async (tx) => {
      const update = await tx.execute(updateStatement, params, options)
      const select = await tx.execute(selectStatement, selectParams, options)

      return [update, select]
    })

    return results?.rows[0] ?? null
  } catch (error) {
    return null
  }
}
