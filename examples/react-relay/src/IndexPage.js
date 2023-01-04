import { Suspense } from 'react'
import { useLazyLoadQuery, usePaginationFragment } from 'react-relay/hooks'
import { graphql } from 'babel-plugin-relay/macro'

function MessagesList(props) {
  const { data, hasNext, loadNext } = usePaginationFragment(
    graphql`
      fragment IndexPageContainer_messages on Query
      @refetchable(queryName: "IndexPageContainerMessagesQuery") {
        messageCollection(first: $first, after: $after)
          @connection(key: "Message__messageCollection") {
          edges {
            cursor
            node {
              id
            }
          }
          pageInfo {
            startCursor
            endCursor
            hasNextPage
            hasPreviousPage
          }
        }
      }
    `,
    props.query
  )

  return (
    <div>
      <pre>{JSON.stringify(data, null, 2)}</pre>
      {hasNext && <button onClick={() => loadNext(1)}>Load more</button>}
    </div>
  )
}

export default function IndexPage() {
  const query = useLazyLoadQuery(
    graphql`
      query IndexPageContainerQuery($first: Int, $after: String) {
        ...IndexPageContainer_messages
      }
    `,
    {
      first: 1,
      after: null
    }
  )

  return (
    <Suspense fallback={<p>Loading...</p>}>
      <h1>Grafbook</h1>
      <MessagesList query={query} />
    </Suspense>
  )
}
