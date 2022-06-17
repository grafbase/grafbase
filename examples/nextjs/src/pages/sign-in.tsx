import type { NextPage } from "next";
import { getSession, signIn } from "next-auth/react";
import type { NextPageContext } from "next";

const Home: NextPage = () => {
  return (
    <div className="flex-1 py-36 flex item-center justify-center animate-show">
      <div className="h-fit shadow-lg rounded-lg border border-gray-200 p-6 space-y-2">
        <h2 className="text-lg font-medium">Todo Example</h2>
        <button
          className="bg-black text-white rounded-md px-2 py-1 hover:bg-gray-800 transition"
          onClick={() => signIn("github")}
        >
          Sign In With GitHub
        </button>
      </div>
    </div>
  );
};

export async function getServerSideProps(context: NextPageContext) {
  const session = await getSession(context);

  if (session) {
    return {
      redirect: {
        destination: "/",
        permanent: false,
      },
    };
  }

  return {
    props: {
      session,
    },
  };
}

export default Home;
