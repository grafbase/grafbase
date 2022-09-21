import type {
  LinksFunction,
  LoaderFunction,
  MetaFunction
} from '@remix-run/node'
import {
  Links,
  LiveReload,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLoaderData
} from '@remix-run/react'
import { PreventFlashOnWrongTheme, ThemeProvider, useTheme } from 'remix-themes'
import Layout from '~/components/layout'
import { themeSessionResolver } from '~/sessions.server'

import styles from './styles/app.css'

export const links: LinksFunction = () => [
  { rel: 'stylesheet', href: styles },
  { rel: 'shortcut icon', href: '/favicon.png' }
]

export const meta: MetaFunction = () => ({
  charset: 'utf-8',
  viewport: 'width=device-width,initial-scale=1'
})

export const loader: LoaderFunction = async ({ request }) => {
  const { getTheme } = await themeSessionResolver(request)
  return {
    theme: getTheme()
  }
}

function App() {
  const data = useLoaderData()
  const [theme] = useTheme()

  return (
    <html lang="en" className={theme ?? ''}>
      <head>
        <Meta />
        <Links />
        <PreventFlashOnWrongTheme ssrTheme={Boolean(data.theme)} />
      </head>
      <Layout>
        <Outlet />
        <ScrollRestoration />
        <Scripts />
        <LiveReload />
      </Layout>
    </html>
  )
}

export default function AppWithProviders() {
  const data = useLoaderData()

  return (
    <ThemeProvider specifiedTheme={data.theme} themeAction="/action/set-theme">
      <App />
    </ThemeProvider>
  )
}
