import { connect, cast } from '@planetscale/database'
import { GraphQLError } from 'graphql'

import { config, options } from '../../lib'

const conn = connect(config)

export default async function ProductsDelete(_, { by }) {
  let statement: string = ''
  let params: (string | number | boolean | {})[] = []

  Object.entries(by).forEach(([field, value]) => {
    if (
      value !== undefined &&
      value !== null &&
      (typeof value === 'string' || typeof value === 'number')
    ) {
      statement = `DELETE FROM Products WHERE ${field} = ?`
      params = [value]
    }
  })

  if (!statement) {
    throw new GraphQLError('ID or Slug must be provided')
  }

  try {
    const results = await conn.execute(statement, params, options)

    console.log(JSON.stringify(results, null, 2))

    if (results.rowsAffected === 1) {
      return { deleted: true }
    }

    return { deleted: false }
  } catch (error) {
    console.log(error)

    return { deleted: false }
  }
}
