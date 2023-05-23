import { createClient } from '@libsql/client/web'

export default async function UsersResolver() {
  const client = createClient({
    url: process.env.LIBSQL_DB_URL,
    authToken: process.env.LIBSQL_DB_AUTH_TOKEN
  })

  try {
    const { rows } = await client.execute('select * from users')

    return rows
  } catch (err) {
    return []
  }
}
