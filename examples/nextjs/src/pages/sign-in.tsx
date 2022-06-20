import type { NextPage } from "next";
import { getSession, signIn } from "next-auth/react";
import type { NextPageContext } from "next";

const Home: NextPage = () => {
  return (
    <div className="flex-1 py-36 flex item-center justify-center animate-show">
      <div className="h-fit shadow-lg rounded-lg border border-gray-200 dark:border-gray-700 p-6 space-y-4">
        <h2 className="text-xl font-semibold text-center">Todo Example</h2>
        <button
          className="flex items-center bg-black dark:bg-zinc-700 text-white rounded-lg px-5 py-3 hover:bg-gray-900"
          onClick={() => signIn("github")}
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            className="mr-2"
          >
            <path
              fillRule="evenodd"
              clipRule="evenodd"
              d="M7.99934 0.26667C3.7295 0.26667 0.266724 3.729 0.266724 8.00026C0.266724 11.4165 2.48235 14.3153 5.55536 15.3384C5.94229 15.4091 6.08328 15.1704 6.08328 14.9652C6.08328 14.7814 6.07663 14.2954 6.07284 13.6501C3.92177 14.1173 3.46792 12.6133 3.46792 12.6133C3.11613 11.7199 2.60911 11.482 2.60911 11.482C1.90697 11.0026 2.66229 11.0121 2.66229 11.0121C3.43849 11.0667 3.84677 11.8092 3.84677 11.8092C4.53657 12.9907 5.65696 12.6494 6.09753 12.4514C6.16779 11.952 6.36765 11.6111 6.58841 11.4179C4.87126 11.2223 3.06582 10.5591 3.06582 7.59578C3.06582 6.7512 3.36728 6.06141 3.86196 5.52068C3.7822 5.32508 3.51683 4.53889 3.93791 3.47404C3.93791 3.47404 4.58688 3.26609 6.06429 4.26638C6.68098 4.095 7.34278 4.00955 8.00029 4.00622C8.65734 4.00955 9.31866 4.095 9.9363 4.26638C11.4128 3.26609 12.0608 3.47404 12.0608 3.47404C12.4828 4.53889 12.2174 5.32508 12.1382 5.52068C12.6338 6.06141 12.9329 6.7512 12.9329 7.59578C12.9329 10.5667 11.1245 11.2205 9.40221 11.4117C9.67946 11.6506 9.92681 12.1225 9.92681 12.8441C9.92681 13.8776 9.9173 14.7117 9.9173 14.9652C9.9173 15.1722 10.0569 15.4129 10.449 15.3374C13.5196 14.3124 15.7334 11.416 15.7334 8.00026C15.7334 3.729 12.2706 0.26667 7.99934 0.26667Z"
              fill="white"
            />
          </svg>
          Sign In with GitHub
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
