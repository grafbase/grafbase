import { gql, useApolloClient, useMutation } from "@apollo/client";
import { CommentAddMutation } from "gql/graphql";
import useViewer from "hooks/use-viewer";
import { FormEvent, useState } from "react";

const ITEM_ADD_COMMENT_MUTATION = gql`
  mutation CommentAdd($content: String!, $authorId: ID!, $itemId: ID!) {
    commentCreate(
      input: {
        content: $content
        author: { link: $authorId }
        item: { link: $itemId }
      }
    ) {
      comment {
        __typename
      }
    }
  }
`;

const ItemAddComment = ({ itemId }: { itemId: string }) => {
  const { viewer } = useViewer();
  const client = useApolloClient();
  const [submitFunction, { loading, error }] = useMutation<CommentAddMutation>(
    ITEM_ADD_COMMENT_MUTATION
  );

  const [content, setContent] = useState("");

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (loading) {
      return;
    }

    const response = await submitFunction({
      variables: {
        content,
        authorId: viewer?.id,
        itemId,
      },
    });

    if (response.data?.commentCreate) {
      setContent("");
      await client.refetchQueries({
        include: ["ItemOne"],
      });
    }
  };

  return (
    <>
      <h2 className="mt-6 text-lg mb-5">Write a comment</h2>
      <form onSubmit={onSubmit} className="space-y-2 text-right">
        <textarea
          className="bg-gray-50 w-full p-4 border focus:outline-black"
          placeholder="Type here"
          value={content}
          onChange={(e) => setContent(e.target.value)}
        />
        {!!error && (
          <div className="bg-red-700 text-white p-4">
            Something went super wrong :(
          </div>
        )}
        <button
          type="submit"
          className={
            "border px-2 py-1 text-gray-700 hover:bg-gray-50" +
            (loading && " animate-pulse bg-gray-200")
          }
        >
          Add comment
        </button>
      </form>
    </>
  );
};

export default ItemAddComment;
