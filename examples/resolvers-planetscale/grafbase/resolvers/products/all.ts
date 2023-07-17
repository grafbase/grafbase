import { connect } from '@planetscale/database'
import { GraphQLError } from 'graphql'

import { config, options } from '../../lib'

const conn = connect(config)

export default async function ProductsAll(_, args) {
  const { first, last, before, after } = args

  try {
    let results

    if (first !== undefined && after !== undefined) {
      results = await conn.execute(
        'SELECT * FROM products WHERE id > ? ORDER BY id ASC LIMIT ?',
        [after, first],
        options
      )
    } else if (last !== undefined && before !== undefined) {
      results = await conn.execute(
        `SELECT * FROM (
          SELECT * FROM products WHERE id < ? ORDER BY id DESC LIMIT ?
        ) AS sub ORDER BY id ASC`,
        [before, last],
        options
      )
    } else if (first !== undefined) {
      results = await conn.execute(
        'SELECT * FROM products ORDER BY id ASC LIMIT ?',
        [first],
        options
      )
    } else if (last !== undefined) {
      results = await conn.execute(
        `SELECT * FROM (
          SELECT * FROM products ORDER BY id DESC LIMIT ?
        ) AS sub ORDER BY id ASC`,
        [last],
        options
      )
    } else {
      throw new GraphQLError(
        'You must provide one of the following arguments: first, last, (first and after), or (last and before)'
      )
    }

    return results?.rows || []
  } catch (error) {
    console.log(error)

    return []
  }
}
