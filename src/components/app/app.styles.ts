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
  backgroundColor: theme.colors.ui_neutral01,
  display: 'grid',
  gridTemplateColumns: `56px 1fr`,
  overflow: 'hidden'
})

// export const StyledToolsContainer = styled('main', {
//   height: '100%',
//   width: '100%',
//   overflow: 'hidden'
// })

// export const StyledToolDisplayWrapper = styled('div', {
//   boxSizing: 'border-box',

//   variants: {
//     isVisible: {
//       true: {
//         visibility: 'visible',
//         opacity: 1,
//         height: '100%',
//         width: '100%'
//       },
//       false: {
//         visibility: 'hidden',
//         opacity: 0,
//         height: 0,
//         width: 0
//       }
//     }
//   }
// })
