import { ReactNode } from 'react'
import Head from 'next/head'
import Logo from 'components/logo'
import ThemeSwitch from 'components/theme-switch'

const Layout = ({ children }: { children: ReactNode }) => {
  return (
    <div>
      <Head>
        <title>Next.js - Todo Example - Grafbase</title>
        <meta
          name="description"
          content="Todo Example leveraging the Grafbase platform"
        />
        <link rel="icon" href="/favicon.png" />
      </Head>
      <div>
        <nav className="flex flex-wrap items-center justify-between p-4 bg-zinc-100 dark:bg-gray-700">
          <div className="flex items-center space-x-6">
            <Logo className="text-black dark:text-white" />
          </div>
          <div className="flex items-center space-x-4">
            <ThemeSwitch />
          </div>
        </nav>
      </div>
      <main className="relative h-[calc(100vh_-_64px)] flex pl-6 pt-6 pr-12 overflow-x-auto dark:bg-zinc-900">
        <div className="fixed left-0 w-6 h-full bg-gradient-to-l from-transparent to-white dark:to-zinc-900" />
        <div className="fixed right-0 w-12 h-full bg-gradient-to-r from-transparent to-white dark:to-zinc-900" />
        {children}
      </main>
    </div>
  )
}

export default Layout
