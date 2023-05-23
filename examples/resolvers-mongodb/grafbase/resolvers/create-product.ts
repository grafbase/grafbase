const baseUrl = process.env.MONGODB_DATA_API_URL
const apiKey = process.env.MONGODB_DATA_API_KEY

export default async function CreateProductResolver(_, { input }) {
  const { name, price } = input

  try {
    const response = await fetch(`${baseUrl}/action/insertOne`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        'api-key': apiKey
      },
      body: JSON.stringify({
        dataSource: 'Cluster0',
        database: 'my-first-database',
        collection: 'products',
        document: {
          name,
          price
        }
      })
    })

    const data = await response.json()

    return {
      id: data.insertedId,
      name,
      price
    }
  } catch (err) {
    console.log(err)

    return null
  }
}
