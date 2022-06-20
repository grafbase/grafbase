import "styles/globals.css";
import type { AppProps } from "next/app";
import { SessionProvider } from "next-auth/react";
import Layout from "components/layout";
import { Provider as UrqlProvider } from "urql";
import { urqlClient } from "graphql/urql";
import { ThemeProvider } from "next-themes";
import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <ThemeProvider attribute="class">
      <SessionProvider session={pageProps.session} refetchInterval={0}>
        <UrqlProvider value={urqlClient}>
          <Layout>
            <Component {...pageProps} />
            <ToastContainer />
          </Layout>
        </UrqlProvider>
      </SessionProvider>
    </ThemeProvider>
  );
}

export default MyApp;
