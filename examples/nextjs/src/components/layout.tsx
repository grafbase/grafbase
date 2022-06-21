import { ReactNode } from "react";
import Head from "next/head";
import { signOut, useSession } from "next-auth/react";
import Logo from "components/logo";
import ThemeSwitch from "components/theme-switch";

const Layout = ({ children }: { children: ReactNode }) => {
  const { data, status } = useSession();
  const loading = status === "loading";

  return (
    <div>
      <Head>
        <title>Next.js - Todo Example - Grafbase</title>
        <meta
          name="description"
          content="Todo Example leveraging the Grafbase platform"
        />
        <link rel="icon" href="/favicon.png" />
      </Head>
      <div>
        <nav className="flex items-center justify-between flex-wrap p-4 bg-zinc-100 dark:bg-zinc-800">
          <div className="flex items-center space-x-6">
            <Logo className="text-black dark:text-white" />
          </div>
          <div className="flex items-center space-x-4">
            {!loading && !!data && (
              <button
                className="border border-gray-400 dark:border-gray-600 text-sm rounded-lg px-2 py-1"
                onClick={() => signOut()}
              >
                Sign Out
              </button>
            )}
            <ThemeSwitch />
          </div>
        </nav>
      </div>
      <main className="relative h-[calc(100vh_-_64px)] flex pl-6 pt-6 pr-12 overflow-x-auto dark:bg-zinc-900">
        <div className="fixed left-0 w-6 h-full bg-gradient-to-l from-transparent to-white dark:to-zinc-900" />
        <div className="fixed right-0 w-12 h-full bg-gradient-to-r from-transparent to-white dark:to-zinc-900" />
        {children}
      </main>
    </div>
  );
};

export default Layout;
