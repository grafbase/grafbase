import { Pathfinder } from '@pathfinder-ide/react'
import '@pathfinder-ide/react/dist/style.css'

import './app.css'

export const App = () => {
  const endpoint = import.meta.env.VITE_GRAFBASE_ENDPOINT || window.GRAPHQL_URL

  const apiKey = import.meta.env.VITE_GRAFBASE_API_KEY || ''

  return (
    <div className="wrap">
      <Pathfinder
        fetcherOptions={{
          endpoint,
          headers: [
            {
              key: 'x-api-key',
              value: apiKey
            }
          ]
        }}
        schemaPollingOptions={{
          enabled: true
        }}
      />
    </div>
  )
}
