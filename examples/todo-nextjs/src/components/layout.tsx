import { ReactNode } from "react";
import Head from "next/head";
import { signOut, useSession } from "next-auth/react";

const Layout = ({ children }: { children: ReactNode }) => {
  const { data, status } = useSession();
  const loading = status === "loading";

  return (
    <div>
      <Head>
        <title>Todo Example - Grafbase</title>
        <meta
          name="description"
          content="Todo Example leveraging on Grafbase platform"
        />
        <link rel="icon" href="/favicon.png" />
      </Head>
      <div>
        <nav className="flex items-center justify-between flex-wrap bg-grafbase p-4 shadow-md">
          <div className="flex items-center space-x-6">
            <h1 className="text-2xl text-white font-medium">Grafbase Todo</h1>
          </div>
          <div className="space-x-2">
            {!loading && !!data && (
              <button
                className="bg-red-600 rounded-md px-2 py-1 text-white"
                onClick={() => signOut()}
              >
                Sign Out
              </button>
            )}
            <a href="https://grafbase.com" target="_blank" rel="noreferrer">
              <button className="bg-green-800 rounded-md px-2 py-1 text-white">
                Visit Grafbase
              </button>
            </a>
          </div>
        </nav>
      </div>{" "}
      <main className="min-h-[calc(100vh_-_133px)] flex p-6 container mx-auto">
        {children}
      </main>
      <footer className="p-4 bg-white border-t border-gray-200 md:flex md:items-center md:justify-between md:p-6 dark:bg-gray-800">
        <span className="text-sm text-gray-500 sm:text-center dark:text-gray-400">
          2022{" "}
          <a href="https://flowbite.com" className="hover:underline">
            Grafbase™
          </a>
        </span>
        <ul className="flex flex-wrap items-center mt-3 text-sm text-gray-500 dark:text-gray-400 sm:mt-0">
          <li>
            <a
              href="https://grafbase.com/docs"
              target="_blank"
              rel="noreferrer"
              className="mr-4 hover:underline md:mr-6 "
            >
              Documentation
            </a>
          </li>
          <li>
            <a
              href="https://grafbase.com/blog"
              target="_blank"
              rel="noreferrer"
              className="mr-4 hover:underline md:mr-6"
            >
              Blog
            </a>
          </li>
          <li>
            <a
              href="https://grafbase.com/templates"
              target="_blank"
              rel="noreferrer"
              className="mr-4 hover:underline md:mr-6"
            >
              Templates
            </a>
          </li>
          <li>
            <a
              href="https://grafbase.com/careers"
              target="_blank"
              rel="noreferrer"
              className="hover:underline"
            >
              Careers
            </a>
          </li>
        </ul>
      </footer>
    </div>
  );
};

export default Layout;
