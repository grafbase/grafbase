// import { Pathfinder, SchemaInitializer } from 'pathfinder'
// import { SchemaDefinition, SchemaDocumentation } from 'pathfinder'
import { darkTheme, theme } from 'ui'

import { useCliApp } from '../../stores'
import { Nav } from '../nav'
import {
  StyledApp, // StyledToolDisplayWrapper,
  // StyledToolsContainer,
  globalStyles
} from './app.styles'

export const App = () => {
  // this global css comes from our stitches setup and is the same as in the next app
  globalStyles()

  // const endpoint =
  //   // eslint-disable-next-line @typescript-eslint/no-explicit-any
  //   import.meta.env.VITE_GRAFBASE_ENDPOINT || window.GRAPHQL_URL

  // const apiKey = import.meta.env.VITE_GRAFBASE_API_KEY || ''

  // const visibleTool = useCliApp(state => state.visibleTool)
  const appTheme = useCliApp(state => state.theme)

  return (
    <StyledApp
      className={appTheme === 'dark' ? darkTheme.className : theme.className}
    >
      <Nav />
      {/* <SchemaInitializer
        fetcherOptions={{
          endpoint,
          http: {
            headers: [['x-api-key', apiKey]]
          },
          sse: {
            protocol: 'EVENT_SOURCE'
          }
        }}
        withPolling={true}
        toRender={
          <StyledToolsContainer>
            <StyledToolDisplayWrapper isVisible={visibleTool === 'Pathfinder'}>
              <Pathfinder withVisualOperationBuilder={true} />
            </StyledToolDisplayWrapper>
            <StyledToolDisplayWrapper
              isVisible={visibleTool === 'SchemaDocumentation'}
            >
              <SchemaDocumentation />
            </StyledToolDisplayWrapper>
            <StyledToolDisplayWrapper
              isVisible={visibleTool === 'SchemaDefinition'}
            >
              <SchemaDefinition />
            </StyledToolDisplayWrapper>
          </StyledToolsContainer>
        }
      /> */}
    </StyledApp>
  )
}
