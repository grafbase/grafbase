import { Icon, IconButton } from 'ui'

import { useCliApp } from '../../stores'
import { ThemeToggle } from '../theme-toggle'
import { StyledGrafbaseLink, StyledNav, StyledNavSection } from './nav.styles'

const setVisibleTool = useCliApp.getState().setVisibleTool

export const Nav = () => {
  const visibleTool = useCliApp(state => state.visibleTool)

  return (
    <StyledNav>
      <StyledNavSection>
        <StyledGrafbaseLink
          href="https://grafbase.com"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Icon name="Grafbase" size="large" />
        </StyledGrafbaseLink>
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
            setVisibleTool({ visibleTool: 'SchemaDocumentation' })
          }}
          iconName="Document"
          isActive={visibleTool === 'SchemaDocumentation'}
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
