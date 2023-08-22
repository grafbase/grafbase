/* eslint-disable turbo/no-undeclared-env-vars */
const username = process.env.COUCHBASE_USERNAME
const password = process.env.COUCHBASE_PASSWORD
const url = process.env.COUCHBASE_URL

export default async function ProductCreateResolver() {
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: 'Basic ' + btoa(`${username}:${password}`)
      },
      body: JSON.stringify({
        statement: `SELECT * FROM store`
      })
    })

    if (!response.ok) {
      throw new Error(await response.text())
    }

    const { results } = await response.json()

    return results?.map(({ store }) => store) ?? []
  } catch (err) {
    console.log(err)
    return []
  }
}
