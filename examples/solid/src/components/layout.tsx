import type { ParentComponent } from 'solid-js'
import { GrafbaseLogo } from '~/components/icons'
import ThemeSwitch from '~/components/theme-switcher'

const Layout: ParentComponent = ({ children }) => {
  return (
    <div class="text-black dark:text-white">
      <nav class="flex flex-wrap items-center justify-between p-4 bg-zinc-100 dark:bg-gray-700">
        <div class="flex items-center space-x-6">
          <GrafbaseLogo class="text-black dark:text-white" />
        </div>
        <div class="flex items-center space-x-4">
          <ThemeSwitch />
        </div>
      </nav>
      <main class="relative h-[calc(100vh_-_64px)] flex pl-6 pt-6 pr-12 overflow-x-auto dark:bg-zinc-900 transition-colors duration-300">
        <div class="fixed left-0 w-6 h-full bg-gradient-to-l from-transparent to-white dark:to-zinc-900" />
        <div class="fixed right-0 w-12 h-full bg-gradient-to-r from-transparent to-white dark:to-zinc-900" />
        {children}
      </main>
    </div>
  )
}

export default Layout
