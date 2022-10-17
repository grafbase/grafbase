import { gql, useMutation } from "@apollo/client";
import { ItemMutation } from "gql/graphql";
import useViewer from "hooks/use-viewer";
import Head from "next/head";
import { useRouter } from "next/router";
import { FormEvent, useState } from "react";

const ITEM_SUBMIT_MUTATION = gql`
  mutation Item($title: String!, $url: URL!, $authorId: ID!) {
    itemCreate(
      input: { title: $title, url: $url, author: { link: $authorId } }
    ) {
      item {
        id
      }
    }
  }
`;

const ItemSubmitPage = () => {
  const { viewer } = useViewer();
  const { push } = useRouter();

  const [submitFunction, { loading, error }] =
    useMutation<ItemMutation>(ITEM_SUBMIT_MUTATION);

  const [form, setForm] = useState<{ url: string; title: string }>({
    url: "",
    title: "",
  });

  const onSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (loading) {
      return;
    }

    const response = await submitFunction({
      variables: {
        title: form.title,
        url: form.url,
        authorId: viewer?.id,
      },
    });

    if (response.data?.itemCreate?.item?.id) {
      push({
        pathname: "/item/[id]",
        query: {
          id: response.data?.itemCreate?.item?.id,
        },
      });
    }
  };

  return (
    <div>
      <Head>
        <title>Submit item | Grafnews</title>
      </Head>
      <h1 className="text-5xl font-bold">Submit</h1>
      <div className="border-b-4 mt-6 max-w-sm border-black" />
      <p className="text-xl mt-4 text-gray-600">
        Share information with the community
      </p>
      <form onSubmit={onSubmit} className="space-y-4 mt-6">
        <div className="flex flex-col space-y-1">
          <label htmlFor="title" className="text-lg font-semibold">
            Title
          </label>
          <input
            id="title"
            name="title"
            type="text"
            placeholder="Type here"
            required
            min="3"
            max="120"
            className="bg-gray-100 p-2 border focus:outline-black"
            onChange={(e) => setForm((f) => ({ ...f, title: e.target.value }))}
          />
        </div>
        <div className="flex flex-col space-y-1">
          <label htmlFor="title" className="text-lg font-semibold">
            Url
          </label>
          <input
            id="Url"
            name="Url"
            type="url"
            placeholder="https://example.com"
            required
            min="3"
            max="300"
            className="bg-gray-100 p-2 border focus:outline-black"
            onChange={(e) => setForm((f) => ({ ...f, url: e.target.value }))}
          />
        </div>
        {!!error && (
          <div className="bg-red-700 text-white p-4">
            Something went super wrong :(
          </div>
        )}
        <div className="pt-2">
          <button
            type="submit"
            disabled={loading}
            className={
              "border bg-black text-lg px-2 py-1 text-white hover:bg-gray-700 " +
              (loading && "animate-pulse")
            }
          >
            Submit
          </button>
        </div>
      </form>
    </div>
  );
};

export default ItemSubmitPage;
