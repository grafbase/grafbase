import { styled, theme } from 'ui'

export const StyledNav = styled('nav', {
  display: 'flex',
  flexDirection: 'column',
  paddingTop: theme.sizes.ui_04,
  paddingBottom: theme.sizes.ui_16,
  justifyContent: 'space-between',
  borderRight: `1px solid ${theme.colors.ui_neutral02}`
})

export const StyledNavSection = styled('nav', {
  display: 'flex',
  flexDirection: 'column',
  gap: theme.sizes.ui_12,
  alignItems: 'center'
})

export const StyledGrafbaseLink = styled('a', {
  position: 'relative',
  paddingBottom: theme.sizes.ui_04,
  height: theme.sizes.ui_48,
  width: theme.sizes.ui_48,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',

  '&::after': {
    content: '',
    position: 'absolute',
    bottom: 0,
    height: 1,
    width: theme.sizes.ui_40,
    backgroundColor: theme.colors.ui_neutral02
  },
  '&:hover': {
    svg: {
      path: {
        fill: theme.colors.ui_neutral07
      }
    }
  }
})
