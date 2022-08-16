import type { ReactNode } from 'react'
import { GrafbaseLogo } from '~/components/icons'
import ThemeSwitch from '~/components/theme-switcher'

const Layout = ({ children }: { children: ReactNode }) => {
  return (
    <body className="text-black dark:text-white">
      <nav className="flex flex-wrap items-center justify-between p-4 bg-zinc-100 dark:bg-gray-700">
        <div className="flex items-center space-x-6">
          <GrafbaseLogo className="text-black dark:text-white" />
        </div>
        <div className="flex items-center space-x-4">
          <ThemeSwitch />
        </div>
      </nav>
      <main className="relative h-[calc(100vh_-_64px)] flex pl-6 pt-6 pr-12 overflow-x-auto dark:bg-zinc-900 transition-colors duration-300">
        <div className="fixed left-0 w-6 h-full bg-gradient-to-l from-transparent to-white dark:to-zinc-900" />
        <div className="fixed right-0 w-12 h-full bg-gradient-to-r from-transparent to-white dark:to-zinc-900" />
        {children}
      </main>
    </body>
  )
}

export default Layout
