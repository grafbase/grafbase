import { Pathfinder } from '@pathfinder-ide/react'
import '@pathfinder-ide/react/dist/style.css'

import { ThemeToggle } from '../theme-toggle'
import { StyledApp, globalStyles } from './app.styles'

export const App = () => {
  // this global css comes from our stitches setup and is the same as in the next app
  globalStyles()

  const endpoint =
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    import.meta.env.VITE_GRAFBASE_ENDPOINT || window.GRAPHQL_URL

  const apiKey = import.meta.env.VITE_GRAFBASE_API_KEY || ''

  return (
    <StyledApp>
      <ThemeToggle />
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
    </StyledApp>
  )
}
