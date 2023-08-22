import { nanoid } from 'nanoid'

/* eslint-disable turbo/no-undeclared-env-vars */
const username = process.env.COUCHBASE_USERNAME
const password = process.env.COUCHBASE_PASSWORD
const url = process.env.COUCHBASE_URL

export default async function ProductsCreateResolver(_, { input }) {
  const id = nanoid()

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: 'Basic ' + btoa(`${username}:${password}`)
      },
      body: JSON.stringify({
        statement: `INSERT INTO store (KEY, VALUE)
          VALUES ("${id}", ${JSON.stringify({ id, ...input })});
        `
      })
    })

    if (!response.ok) {
      throw new Error(await response.text())
    }

    return {
      id,
      ...input
    }
  } catch (err) {
    console.log(err)
    return null
  }
}
