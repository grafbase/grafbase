import { Icon, IconButton } from 'ui'

import { useCliApp } from '../../stores'
import { ThemeToggle } from '../theme-toggle'
import { StyledGrafbaseIcon, StyledNav, StyledNavSection } from './nav.styles'

const setVisibleTool = useCliApp.getState().setVisibleTool

export const Nav = () => {
  const visibleTool = useCliApp(state => state.visibleTool)

  return (
    <StyledNav>
      <StyledNavSection>
        <StyledGrafbaseIcon>
          <Icon name="Grafbase" size="large" />
        </StyledGrafbaseIcon>
        <IconButton
          action={() => {
            setVisibleTool({ visibleTool: 'Pathfinder' })
          }}
          iconName="Compass"
          isActive={visibleTool === 'Pathfinder'}
          title="View Pathfinder"
          size={'large'}
        />
        <IconButton
          action={() => {
            setVisibleTool({ visibleTool: 'SchemaDocumentationViewer' })
          }}
          iconName="Document"
          isActive={visibleTool === 'SchemaDocumentationViewer'}
          title="View Schema Documentation"
          size={'large'}
        />
        <IconButton
          action={() => {
            setVisibleTool({ visibleTool: 'SchemaDefinition' })
          }}
          iconName="GraphQL"
          isActive={visibleTool === 'SchemaDefinition'}
          title="View Schema Definition"
          size={'large'}
        />
      </StyledNavSection>
      <StyledNavSection>
        <ThemeToggle />
      </StyledNavSection>
    </StyledNav>
  )
}
