import { gql, useApolloClient, useMutation, useQuery } from "@apollo/client";
import { SignedIn } from "@clerk/nextjs";
import Img from "components/img";
import ItemAddComment from "components/item-add-comment";
import ItemComment from "components/item-comment";
import ItemVotes from "components/item-votes";
import formatDistanceToNow from "date-fns/formatDistanceToNow";
import { ItemOneQuery } from "gql/graphql";
import useViewer from "hooks/use-viewer";
import Head from "next/head";
import { useRouter } from "next/router";

const ITEM_QUERY = gql`
  query ItemOne($id: ID!) {
    item(id: $id) {
      id
      title
      comments(first: 100) {
        edges {
          node {
            id
            content
            #            createdAt
            author {
              id
              name
              imageUrl
            }
          }
        }
      }
      votes(first: 100) {
        edges {
          node {
            id
            positive
            user {
              id
            }
          }
        }
      }
      author {
        id
        name
        imageUrl
      }
      url
      createdAt
    }
  }
`;

const ITEM_DELETE_MUTATION = gql`
  mutation ItemOneDelete($id: ID!) {
    itemDelete(id: $id) {
      deletedId
    }
  }
`;

const ItemIdPage = () => {
  const client = useApolloClient();
  const { query, replace } = useRouter();
  const { viewer } = useViewer();
  const { data, loading, error } = useQuery<ItemOneQuery>(ITEM_QUERY, {
    variables: { id: query.id },
  });
  const [deleteMutation] = useMutation(ITEM_DELETE_MUTATION);

  if (loading || !data?.item) {
    return null;
  }

  const { id, title, comments, createdAt, url, votes, author } = data?.item;

  const isSessionUserItem = author.id === viewer?.id;

  const onDelete = () => {
    if (confirm("Are you sure you want to delete this item?")) {
      deleteMutation({ variables: { id } }).then(() => replace("/"));
    }
  };

  return (
    <div>
      <Head>
        <title>{title} | Grafnews</title>
      </Head>
      <div className="flex">
        <div className="flex flex-col border border-black">
          <ItemVotes itemId={id} votes={votes} />
        </div>
        <div className="pl-4 flex-1">
          <h1 className="text-5xl font-bold">{title}</h1>
          <div className="mt-4">
            <div className="bg-gray-100 p-4 text-xl  text-gray-800">
              <a href={url} target="_blank" rel="noreferrer">
                {url}
              </a>
            </div>
          </div>
          <div className="flex justify-end space-x-2 items-center mt-4">
            {isSessionUserItem && (
              <button
                onClick={onDelete}
                className="text-gray-700 hover:bg-red-200 px-2 border"
              >
                Delete
              </button>
            )}
            <span className="text-gray-500">
              <time className="font-semibold text-gray-700">
                {!!createdAt &&
                  formatDistanceToNow(Date.parse(createdAt), {
                    addSuffix: true,
                  })}
              </time>{" "}
              by {author.name}
            </span>
            <Img src={author.imageUrl} alt={author.name} className="h-7 w-7" />
          </div>
        </div>
      </div>
      <hr className="mt-6" />
      <div>
        <SignedIn>
          <ItemAddComment itemId={id} />
        </SignedIn>
        <div>
          <h2 className="mt-6 text-lg mb-5">
            {comments?.edges?.length
              ? `Comments (${comments?.edges?.length})`
              : "No comments yet"}
          </h2>
          <div className="space-y-4">
            {comments?.edges?.map((edge) => {
              if (!edge?.node) {
                return null;
              }

              return <ItemComment key={edge.node.id} {...edge.node} />;
            })}
          </div>
        </div>
      </div>
    </div>
  );
};

export default ItemIdPage;
