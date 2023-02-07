import { Theme, useTheme } from '@graphiql/react'
import { useLayoutEffect } from 'react'

export const ThemeProvider = ({
  theme,
  children
}: {
  theme?: Theme
  children: any
}) => {
  const { setTheme } = useTheme()
  useLayoutEffect(() => {
    if (theme) {
      setTheme(theme)
    }
  }, [theme, setTheme])
  return <>{children}</>
}
