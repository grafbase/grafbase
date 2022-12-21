import React from 'react'
import ReactDOM from 'react-dom/client'
import { Provider } from 'urql'
import App from './App'
import { AuthProvider } from './auth'
import { client } from './client'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <AuthProvider>
      <Provider value={client}>
        <App />
      </Provider>
    </AuthProvider>
  </React.StrictMode>
)
