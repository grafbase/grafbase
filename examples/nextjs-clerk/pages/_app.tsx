import {
  ClerkLoaded,
  ClerkProvider,
  SignedIn,
  SignedOut,
  RedirectToSignIn
} from '@clerk/nextjs'
import type { AppProps } from 'next/app'

const publicPages = ['/', '/sign-in', '/sign-up']

function MyApp({ Component, pageProps, router }: AppProps) {
  return (
    <ClerkProvider {...pageProps}>
      <ClerkLoaded>
        {publicPages.includes(router.pathname) ? (
          <Component {...pageProps} />
        ) : (
          <>
            <SignedIn>
              <Component {...pageProps} />
            </SignedIn>
            <SignedOut>
              <RedirectToSignIn />
            </SignedOut>
          </>
        )}
      </ClerkLoaded>
    </ClerkProvider>
  )
}

export default MyApp
