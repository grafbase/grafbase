import { createEffect, createSignal } from 'solid-js'
import { MoonIcon, SunIcon } from '~/components/icons'

enum Theme {
  DARK = 'dark',
  LIGHT = 'light'
}

const initializeTheme = () => {
  let theme: Theme = Theme.LIGHT
  if (typeof localStorage !== 'undefined' && localStorage.getItem('theme')) {
    theme = localStorage.getItem('theme') as Theme
  } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    theme = Theme.DARK
  } else {
    theme = Theme.LIGHT
  }
  return theme
}

const ThemeSwitch = () => {
  const [theme, setTheme] = createSignal<Theme>(initializeTheme())

  createEffect(() => {
    const root = document.documentElement
    if (theme() === Theme.LIGHT) {
      root.classList.remove(Theme.DARK)
      localStorage.setItem('theme', Theme.LIGHT)
    } else {
      root.classList.add(Theme.DARK)
      localStorage.setItem('theme', Theme.DARK)
    }
  })

  const toggleTheme = () =>
    setTheme((t) => (t === Theme.DARK ? Theme.LIGHT : Theme.DARK))

  return (
    <label
      for="theme-switcher"
      class="relative inline-flex items-center cursor-pointer"
    >
      <input
        id="theme-switcher"
        type="checkbox"
        checked={theme() === Theme.DARK}
        onChange={toggleTheme}
        class="sr-only peer"
      />
      <SunIcon class="absolute z-10 w-4 h-4 left-2 dark:text-gray-400" />
      <MoonIcon class="absolute z-10 w-4 h-4 text-gray-400 right-2 dark:text-white" />
      <span class="w-14 h-8 bg-[#38361F] peer-focus:outline-none peer-focus:ring-0 rounded-full dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[4px] after:left-[4px] after:bg-yellow-400 after:dark:bg-blue-500 after:rounded-full after:h-6 after:w-6 after:transition-all dark:border-gray-600 peer-checked:bg-[#303853] peer-checked:dark:bg-blue-200" />
    </label>
  )
}

export default ThemeSwitch
