import { createClient } from '@libsql/client/web'

const client = createClient({
  url: process.env.LIBSQL_DB_URL,
  authToken: process.env.LIBSQL_DB_AUTH_TOKEN
})

export default async function CreateUserResolver(_, { name }) {
  await client.execute({
    sql: 'insert into users values (?)',
    args: [name]
  })

  return { name }
}
