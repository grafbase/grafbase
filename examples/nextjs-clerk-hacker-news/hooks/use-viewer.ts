import { gql, useQuery } from '@apollo/client'
import { useAuth, useSession } from '@clerk/nextjs'
import { ViewerQuery } from 'gql/graphql'

const VIEWER_QUERY = gql`
  query Viewer($email: Email!) {
    user(by: { email: $email }) {
      id
      name
      email
      imageUrl
      createdAt
      items(first: 3) {
        edges {
          __typename
        }
      }
    }
  }
`

const useViewer = () => {
  const { isSignedIn } = useAuth()
  const { session } = useSession()
  const { loading, data } = useQuery<ViewerQuery>(VIEWER_QUERY, {
    variables: { email: session?.user?.emailAddresses[0].emailAddress },
    skip: !isSignedIn
  })

  return {
    viewer: data?.user,
    loading
  }
}

export default useViewer
