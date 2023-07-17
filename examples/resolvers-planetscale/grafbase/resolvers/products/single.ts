import { connect } from '@planetscale/database'
import { GraphQLError } from 'graphql'

import { config, options } from '../../lib'

const conn = connect(config)

export default async function ProductsSingle(_, { by }) {
  let results

  try {
    if (by.id !== undefined && by.slug !== undefined) {
      throw new GraphQLError('Only one of ID or Slug should be provided')
    } else if (by.id !== undefined) {
      results = await conn.execute(
        'SELECT * FROM products WHERE id = ? LIMIT 1',
        [by.id],
        options
      )
    } else if (by.slug !== undefined) {
      results = await conn.execute(
        'SELECT * FROM products WHERE slug = ? LIMIT 1',
        [by.slug],
        options
      )
    } else {
      throw new GraphQLError('ID or Slug must be provided')
    }

    return results?.rows[0] ?? null
  } catch (error) {
    console.log(error)

    return null
  }
}
