import { useSignIn, useSignUp } from "@clerk/nextjs";
import { withServerSideAuth } from "@clerk/nextjs/ssr";
import { AuthenticateWithRedirectParams } from "@clerk/types/dist/oauth";
import { GetServerSideProps } from "next";
import Head from "next/head";

const LoginPage = () => {
  const { signUp } = useSignUp();
  const { signIn } = useSignIn();

  // We have no way to tell if an user already exists at this point, so we "rely" on localStorage
  const onGithubClick = () => {
    const hasLoggedIn = localStorage.getItem("hasLoggedIn");
    const params: AuthenticateWithRedirectParams = {
      strategy: "oauth_github",
      redirectUrl: `/callback`,
      redirectUrlComplete: `/callback-login`,
    };
    if (hasLoggedIn) {
      signIn?.authenticateWithRedirect(params);
    } else {
      signUp?.authenticateWithRedirect(params);
    }
  };

  return (
    <div className="min-h-screen flex flex-col items-center justify-center -mt-16">
      <Head>
        <title>Login | Grafnews</title>
      </Head>
      <div className="border border-gray-600 max-w-lg mx-auto border-b-4">
        <div className="px-6 py-8 bg-gray-200">
          <h1 className="text-4xl font-bold text-center">Join Grafnews</h1>
        </div>
        <div className="px-6 pb-8">
          <button
            onClick={onGithubClick}
            className="flex items-center justify-center space-x-4 border-black border px-4 w-full py-3 mt-6 text-2xl  bg-black text-white hover:bg-indigo-800"
          >
            <svg fill="currentColor" viewBox="0 0 24 24" className="w-8 h-8">
              <path
                fillRule="evenodd"
                d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                clipRule="evenodd"
              />
            </svg>
            <span>Continue With GitHub</span>
          </button>
        </div>
      </div>
      <div className="text-center mt-6">
        Auth powered by{" "}
        <a
          href="https://clerk.dev"
          target="_blank"
          rel="noreferrer"
          className="font-semibold"
        >
          Clerk
        </a>{" "}
        ;)
      </div>
    </div>
  );
};

export const getServerSideProps: GetServerSideProps = withServerSideAuth(
  async (context) => {
    const userId = context.req.auth.userId;

    if (userId) {
      return {
        redirect: {
          permanent: true,
          destination: `/`,
        },
      };
    }

    return {
      props: {},
    };
  }
);

export default LoginPage;
