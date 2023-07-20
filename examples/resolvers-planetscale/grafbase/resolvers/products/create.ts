import { connect } from '@planetscale/database'

import { config, options } from '../../lib'

const conn = connect(config)

export default async function ProductsCreate(_, { input }) {
  const fields: string[] = []
  const placeholders: string[] = []
  const values: (string | number | boolean)[] = []

  Object.entries(input).forEach(([field, value]) => {
    if (
      value !== undefined &&
      value !== null &&
      (typeof value === 'string' ||
        typeof value === 'number' ||
        typeof value === 'boolean')
    ) {
      fields.push(`\`${field}\``) // field names should be escaped
      placeholders.push('?')
      values.push(value)
    }
  })

  const statement = `INSERT INTO products (${fields.join(
    ', '
  )}) VALUES (${placeholders.join(', ')})`

  try {
    const { insertId } = await conn.execute(statement, values, options)

    return {
      id: insertId,
      ...input
    }
  } catch (error) {
    console.log(error)

    return null
  }
}
