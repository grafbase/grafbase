import { AuthenticateWithRedirectCallback } from "@clerk/nextjs";
import { withServerSideAuth } from "@clerk/nextjs/ssr";
import LogoAnimated from "components/logo-animated";
import { GetServerSideProps } from "next";
import Head from "next/head";

// This screen has another callback to callback-login because me must insert or update the user in order
// for the viewer to work
const CallbackPage = () => {
  return (
    <>
      <Head>
        <title>Loading Session | Grafnews</title>
      </Head>
      <div className="flex items-center justify-center min-h-screen">
        <div className="border border-black pt-6 pb-4 px-6 bg-gray-50 border-b-4">
          <LogoAnimated />
        </div>
        <AuthenticateWithRedirectCallback redirectUrl="/callback-login" />
      </div>
    </>
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

export default CallbackPage;
