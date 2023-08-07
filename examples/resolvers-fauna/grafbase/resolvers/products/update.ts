import { Client, fql, FaunaError } from 'fauna'
import { GraphQLError } from 'graphql'

const client = new Client()

export default async function ProductsUpdate(_, { by, input }) {
  const { id } = by

  if (Object.entries(input).length === 0) {
    throw new GraphQLError('At least one field to update must be provided.')
  }

  try {
    const documentQuery = fql`
    products.byId(${id}).update(${input}) {
      id,
      name,
      price
    }
  `

    const { data } = await client.query(documentQuery)

    console.log(JSON.stringify(data, null, 2))

    return data
  } catch (error) {
    if (error instanceof FaunaError) {
      throw new GraphQLError(error?.message)
    }

    return null
  }
}
