import { Client, fql, FaunaError } from 'fauna'

const client = new Client()

export default async function ProductsCreate(_, { input }) {
  try {
    const documentQuery = fql`
      products.create(${input}) {
        id,
        name,
        price
      }
    `

    const { data } = await client.query(documentQuery)

    return data
  } catch (error) {
    if (error instanceof FaunaError) {
      console.log(error)
    }

    return null
  }
}
