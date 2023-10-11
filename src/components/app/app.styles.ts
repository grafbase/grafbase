import { globalCss, styled, theme } from 'ui'

export const globalStyles = globalCss({
  html: {
    height: '100%',
    width: '100%'
  },
  body: {
    margin: 0,
    padding: 0,
    height: '100%',
    width: '100%',
    fontFamily: theme.fonts.sans,

    '& #root': {
      height: '100%',
      width: '100%'
    }
  }
})

export const StyledApp = styled('div', {
  height: '100%',
  width: '100%',
  overflow: 'hidden'
})
