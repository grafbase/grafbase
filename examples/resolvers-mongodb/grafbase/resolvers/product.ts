// @ts-expect-error
const baseUrl = process.env.MONGODB_DATA_API_URL
// @ts-expect-error
const apiKey = process.env.MONGODB_DATA_API_KEY

export default async function ProductResolver(_, { id }) {
  try {
    const response = await fetch(`${baseUrl}/action/findOne`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        'api-key': apiKey
      },
      body: JSON.stringify({
        dataSource: 'Cluster0',
        database: 'my-first-database',
        collection: 'products',
        filter: {
          _id: {
            $oid: id
          }
        }
      })
    })

    const { document } = await response.json()

    if (document === null) return null

    return {
      id,
      ...document
    }
  } catch (err) {
    console.log(err)
    return null
  }
}