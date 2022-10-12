import { useAuth } from "@clerk/nextjs";
import Img from "components/img";
import formatDistanceToNow from "date-fns/formatDistanceToNow";
import useViewer from "hooks/use-viewer";
import Head from "next/head";

const UserIdPage = () => {
  const { signOut } = useAuth();
  const { viewer } = useViewer();

  if (!viewer) {
    return null;
  }

  return (
    <div>
      <Head>
        <title>{viewer?.name || "User"} | Grafnews</title>
      </Head>
      <div className="flex justify-between items-center">
        <div className="flex items-center space-x-4">
          <Img alt="User image" src={viewer?.imageUrl} className="w-14 h-14" />
          <h1 className="text-5xl font-bold">{viewer?.name}</h1>
        </div>
        <button
          onClick={() => signOut()}
          className="px-2 py-1 bg-red-700 text-white text-xl"
        >
          Logout
        </button>
      </div>
      <div className="border-b-4 mt-6 max-w-sm border-black" />
      <p className="text-xl mt-4 text-gray-600">
        {viewer?.email} | Joined{" "}
        <time>
          {!!viewer?.createdAt &&
            formatDistanceToNow(Date.parse(viewer?.createdAt), {
              addSuffix: true,
            })}
        </time>
      </p>
      {/*<p className="text-xl mt-4 text-gray-600"></p>*/}
      {/*<h3 className="mt-8 text-2xl font-semibold">Last 3 items</h3>*/}
      {/*<div className="space-y-4 mt-6">*/}
      {/*  {viewer?.items?.edges?.map(*/}
      {/*    (edge) => !!edge && <ItemList key={edge.node.id} {...edge.node} />*/}
      {/*  )}*/}
      {/*</div>*/}
    </div>
  );
};

export default UserIdPage;
