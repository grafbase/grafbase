import { Client, fql, FaunaError } from 'fauna'

const client = new Client()

export default async function ProductsAll() {
  try {
    const documentQuery = fql`
      products.all() {
        id,
        name,
        price
      }
    `

    const { data } = await client.query(documentQuery)

    return data?.data || []
  } catch (error) {
    if (error instanceof FaunaError) {
      console.log(error)
    }

    return []
  }
}
