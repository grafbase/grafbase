import NextHead from "next/head";
import { PropsWithChildren } from "react";

const Head = ({ children }: PropsWithChildren) => {
  return (
    <NextHead>
      <title>Grafnews</title>
      <meta
        name="description"
        content="Nextjs Hacker News example with Clerk and Apollo"
      />
      <link rel="icon" href="/public/favicon.ico" />
      {children}
    </NextHead>
  );
};

export default Head;
