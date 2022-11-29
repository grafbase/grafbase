import { ApolloClient, ApolloProvider } from '@apollo/client'
import {
  ClerkProvider,
  RedirectToSignIn,
  SignedIn,
  SignedOut,
  useAuth
} from '@clerk/clerk-react'
import React, { PropsWithChildren, useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { createApolloLink, initializeApolloClient } from './client'

const ApolloProviderWrapper = ({ children }: PropsWithChildren) => {
  const { getToken } = useAuth()
  const [isLoading, setIsLoading] = useState(true)
  const [client, setClient] = useState<ApolloClient<any>>()

  useEffect(() => {
    const init = async () => {
      const token = (await getToken()) ?? ''
      const apolloClient = initializeApolloClient(createApolloLink(token))
      setClient(apolloClient)
      setIsLoading(false)
    }
    init()
  }, [])

  if (isLoading) return <p>Loading...</p>

  return <ApolloProvider client={client}>{children}</ApolloProvider>
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ClerkProvider frontendApi={import.meta.env.VITE_CLERK_FRONTEND_API}>
      <SignedIn>
        <ApolloProviderWrapper>
          <App />
        </ApolloProviderWrapper>
      </SignedIn>
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    </ClerkProvider>
  </React.StrictMode>
)
