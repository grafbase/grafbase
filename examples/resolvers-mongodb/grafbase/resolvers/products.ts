const baseUrl = process.env.MONGODB_DATA_API_URL
const apiKey = process.env.MONGODB_DATA_API_KEY

export default async function ProductsResolver(_, { limit }) {
  try {
    const response = await fetch(`${baseUrl}/action/find`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        'api-key': apiKey
      },
      body: JSON.stringify({
        dataSource: 'Cluster0',
        database: 'my-first-database',
        collection: 'products',
        limit
      })
    })

    const data = await response.json()

    return data?.documents?.map(({ _id: id, name, price }) => ({
      id,
      name,
      price
    }))
  } catch (err) {
    console.log(err)
    return null
  }
}
