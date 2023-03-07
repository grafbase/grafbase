import { gql, useQuery } from '@apollo/client'

const POSTS_QUERY = gql`
  query Comments($cursor: String) {
    postCollection(first: 1, after: $cursor) {
      edges {
        node {
          id
          title
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
`

export default function App() {
  const { data, loading, fetchMore } = useQuery(POSTS_QUERY)

  if (loading) return <p>Loading posts...</p>

  const nodes = data.postCollection.edges.map((edge) => edge.node)
  const pageInfo = data.postCollection.pageInfo

  return (
    <div>
      <h1>Relay Style Pagination</h1>
      <ul>
        {nodes.map(({ id, title }) => (
          <li key={id}>{title}</li>
        ))}
      </ul>
      {pageInfo.hasNextPage && (
        <button
          onClick={() =>
            fetchMore({
              variables: {
                cursor: pageInfo.endCursor
              }
            })
          }
        >
          Load more
        </button>
      )}
    </div>
  )
}
