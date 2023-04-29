import { Pathfinder } from 'pathfinder'
import { useSchema } from 'pathfinder/src/stores'
import { SchemaReference } from 'schema-documentation-viewer'
import { SchemaDefinition, darkTheme, theme } from 'ui'

import { useCliApp } from '../../stores'
import { Nav } from '../nav'
import {
  StyledApp,
  StyledToolDisplayWrapper,
  StyledToolsContainer,
  globalStyles
} from './app.styles'

export const App = () => {
  // this global css comes from our stitches setup and is the same as in the next app
  globalStyles()

  const endpoint =
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    import.meta.env.VITE_GRAFBASE_ENDPOINT || window.GRAPHQL_URL

  const apiKey = import.meta.env.VITE_GRAFBASE_API_KEY || ''

  const visibleTool = useCliApp(state => state.visibleTool)
  const appTheme = useCliApp(state => state.theme)

  const schema = useSchema(state => state.schema)

  return (
    <StyledApp
      className={appTheme === 'dark' ? darkTheme.className : theme.className}
    >
      <Nav />
      <StyledToolsContainer>
        <StyledToolDisplayWrapper isVisible={visibleTool === 'Pathfinder'}>
          <Pathfinder
            fetcherOptions={{
              endpoint,
              http: {
                headers: {
                  'x-api-key': apiKey
                }
              },
              sse: {
                protocol: 'EVENT_SOURCE'
              }
            }}
            withPolling={true}
            withVisualOperationBuilder={true}
          />
        </StyledToolDisplayWrapper>
        <StyledToolDisplayWrapper
          isVisible={visibleTool === 'SchemaDocumentationViewer'}
        >
          {schema && !('error' in schema) && (
            <SchemaReference
              schemaOrConnectionDetails={
                schema || {
                  endpoint,
                  headers: { 'x-api-key': apiKey }
                }
              }
            />
          )}
        </StyledToolDisplayWrapper>
        <StyledToolDisplayWrapper
          isVisible={visibleTool === 'SchemaDefinition'}
        >
          {schema && !('error' in schema) && (
            <SchemaDefinition schema={schema} />
          )}
        </StyledToolDisplayWrapper>
      </StyledToolsContainer>
    </StyledApp>
  )
}
