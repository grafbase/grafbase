// @ts-expect-error
const baseUrl = process.env.MONGODB_DATA_API_URL
// @ts-expect-error
const apiKey = process.env.MONGODB_DATA_API_KEY

export default async function DeleteProductResolver(_, { id }) {
  try {
    const response = await fetch(`${baseUrl}/action/deleteOne`, {
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

    const { deletedCount }  = await response.json()

    return !!deletedCount
    
  } catch (err) {
    console.log(err)
    return false
  }
}