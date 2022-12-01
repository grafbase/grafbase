import { ApolloClient, ApolloProvider } from '@apollo/client'
import React, { PropsWithChildren, useEffect, useState } from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import { AuthProvider, useAuth } from './auth'
import { createApolloLink, initializeApolloClient } from './client'

const ApolloProviderWrapper = ({ children }: PropsWithChildren) => {
  const { token } = useAuth()
  const [isLoading, setIsLoading] = useState(true)
  const [client, setClient] = useState({} as ApolloClient<any>)

  useEffect(() => {
    if (token) {
      const apolloClient = initializeApolloClient(createApolloLink(token))
      setClient(apolloClient)
      setIsLoading(false)
    }
  }, [token])

  if (isLoading) return <p>Loading...</p>

  return <ApolloProvider client={client}>{children}</ApolloProvider>
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <AuthProvider>
      <ApolloProviderWrapper>
        <App />
      </ApolloProviderWrapper>
    </AuthProvider>
  </React.StrictMode>
)
