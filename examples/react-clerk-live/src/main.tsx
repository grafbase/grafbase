import { ClerkProvider, SignedIn, SignedOut, UserButton, useUser, RedirectToSignIn } from '@clerk/clerk-react'
import React from 'react'
import ReactDOM from 'react-dom/client'
import { Provider } from 'urql'
import App from './App'
import { client } from './client'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ClerkProvider frontendApi={import.meta.env.VITE_CLERK_FRONTEND_API}>
      <SignedIn>
        <Provider value={client}>
          <App />
        </Provider>
      </SignedIn>
      <SignedOut>
        <RedirectToSignIn />
      </SignedOut>
    </ClerkProvider>
  </React.StrictMode>
)
