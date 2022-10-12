import { gql, useQuery } from "@apollo/client";
import { useAuth, useSession } from "@clerk/nextjs";
import { ViewerQuery } from "gql/graphql";
import { useMemo } from "react";

const VIEWER_QUERY = gql`
  query Viewer {
    userCollection(first: 100) {
      edges {
        node {
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
    }
  }
`;

const useViewer = () => {
  const { isSignedIn } = useAuth();
  const { session } = useSession();
  const { loading, data } = useQuery<ViewerQuery>(VIEWER_QUERY, {
    skip: !isSignedIn,
  });

  const viewer = useMemo(() => {
    return data?.userCollection?.edges?.find(
      (edge) =>
        edge &&
        edge.node?.email === session?.user?.emailAddresses[0].emailAddress
    )?.node;
  }, [data?.userCollection?.edges, session?.user?.emailAddresses]);

  return {
    loading,
    viewer,
  };
};

export default useViewer;
