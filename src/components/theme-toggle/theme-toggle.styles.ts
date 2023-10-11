import { styled, theme, uiCommon } from 'ui'

export const StyledThemeToggle = styled('button', {
  ...uiCommon.buttonReset,
  display: 'flex',
  height: theme.sizes.ui_20,
  width: theme.sizes.ui_20,
  position: 'fixed',
  bottom: 24,
  left: 20,

  svg: {
    path: {
      fill: theme.colors.ui_neutral05
    }
  },

  '&:hover': {
    svg: {
      path: {
        fill: theme.colors.ui_neutral06
      }
    }
  }
})
