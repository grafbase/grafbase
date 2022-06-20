import "styles/globals.css";
import type { AppProps } from "next/app";
import { SessionProvider } from "next-auth/react";
import Layout from "components/layout";
import { Provider as UrqlProvider } from "urql";
import { urqlClient } from "graphql/urql";
import { ThemeProvider } from "next-themes";

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <ThemeProvider attribute="class">
      <SessionProvider session={pageProps.session} refetchInterval={0}>
        <UrqlProvider value={urqlClient}>
          <Layout>
            <Component {...pageProps} />
          </Layout>
        </UrqlProvider>
      </SessionProvider>
    </ThemeProvider>
  );
}

export default MyApp;
