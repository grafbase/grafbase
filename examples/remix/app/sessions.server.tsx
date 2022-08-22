import { createCookieSessionStorage } from '@remix-run/node'
import { createThemeSessionResolver } from 'remix-themes'

const sessionStorage = createCookieSessionStorage({
  cookie: {
    name: 'remix-themes',
    secure: true,
    sameSite: 'lax',
    secrets: ['s3cr3t'],
    path: '/',
    httpOnly: true
  }
})

export const themeSessionResolver = createThemeSessionResolver(sessionStorage)
