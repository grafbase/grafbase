import { gql, useApolloClient, useMutation } from "@apollo/client";
import {
  ItemsListQuery,
  ItemVoteMutation,
  ItemVoteUpdateMutation,
} from "gql/graphql";
import useViewer from "hooks/use-viewer";

const ITEM_VOTE_MUTATION = gql`
  mutation ItemVote($vote: Boolean!, $authorId: ID!, $itemId: ID!) {
    voteCreate(
      input: {
        positive: $vote
        user: { link: $authorId }
        item: { link: $itemId }
      }
    ) {
      vote {
        __typename
      }
    }
  }
`;

const ITEM_VOTE_UPDATE_MUTATION = gql`
  mutation ItemVoteUpdate($id: ID!, $vote: Boolean!) {
    voteUpdate(id: $id, input: { positive: $vote }) {
      vote {
        __typename
      }
    }
  }
`;

const ItemVotes = (props: {
  itemId: string;
  votes?: NonNullable<
    NonNullable<NonNullable<ItemsListQuery["itemCollection"]>["edges"]>[0]
  >["node"]["votes"];
}) => {
  const { votes, itemId } = props;
  const { viewer } = useViewer();
  const client = useApolloClient();
  const [voteFunction, { loading: voteLoading }] =
    useMutation<ItemVoteMutation>(ITEM_VOTE_MUTATION);

  const [voteUpdateFunction, { loading: voteUpdateLoading }] =
    useMutation<ItemVoteUpdateMutation>(ITEM_VOTE_UPDATE_MUTATION);

  const userVote = votes?.edges?.find((edge) => {
    if (!edge?.node) {
      return false;
    }

    return edge?.node?.user?.id === viewer?.id;
  });

  const aggregate = votes?.edges
    ?.map((edge) => {
      if (!edge?.node) {
        return 0;
      }

      return edge?.node?.positive ? 1 : -1;
    })
    .reduce<number>((a, b) => a + b, 0);

  const onClick = async (vote: boolean) => {
    if (voteLoading || voteUpdateLoading) {
      return null;
    }

    if (userVote) {
      await voteUpdateFunction({
        variables: {
          id: userVote.node?.id,
          vote,
        },
      });
    } else {
      await voteFunction({
        variables: {
          vote,
          authorId: viewer?.id,
          itemId,
        },
      });
    }

    client.refetchQueries({
      include: ["ItemOne", "ItemsList"],
    });
  };

  return (
    <>
      <button
        onClick={() => onClick(true)}
        className={
          "flex flex-1 items-center justify-center p-2 hover:bg-green-500 hover:text-white" +
          (userVote?.node?.positive && " bg-green-500 pointer-events-none")
        }
      >
        <svg
          viewBox="0 0 24 24"
          className="w-6 h-6"
          stroke="currentColor"
          strokeWidth="2"
          fill="none"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="m18 15-6-6-6 6"></path>
        </svg>
      </button>
      <div className="flex flex-1 items-center justify-center p-2 bg-black text-white font-bold text-lg border-y border-black">
        {aggregate || 0}
      </div>
      <button
        onClick={() => onClick(false)}
        className={
          "flex flex-1 items-center justify-center p-2 hover:bg-red-500 hover:text-white" +
          (userVote?.node?.positive === false &&
            " bg-red-500 pointer-events-none")
        }
      >
        <svg
          viewBox="0 0 24 24"
          className="w-6 h-6"
          stroke="currentColor"
          strokeWidth="2"
          fill="none"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="m6 9 6 6 6-6"></path>
        </svg>
      </button>
    </>
  );
};

export default ItemVotes;
