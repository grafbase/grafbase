import { Theme, useTheme } from 'remix-themes'
import { MoonIcon, SunIcon } from '~/components/icons'

const ThemeSwitch = () => {
  const [theme, setTheme] = useTheme()
  const toggleTheme = () =>
    setTheme((prevTheme) =>
      prevTheme === Theme.DARK ? Theme.LIGHT : Theme.DARK
    )

  return (
    <label
      htmlFor="theme-switcher"
      className="relative inline-flex items-center cursor-pointer"
    >
      <input
        id="theme-switcher"
        type="checkbox"
        defaultChecked={theme === Theme.DARK}
        onChange={toggleTheme}
        className="sr-only peer"
      />
      <SunIcon className="absolute z-10 w-4 h-4 left-2 dark:text-gray-400" />
      <MoonIcon className="absolute z-10 w-4 h-4 text-gray-400 right-2 dark:text-white" />
      <span className="w-14 h-8 bg-[#38361F] peer-focus:outline-none peer-focus:ring-0 rounded-full dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[4px] after:left-[4px] after:bg-yellow-400 after:dark:bg-blue-500 after:rounded-full after:h-6 after:w-6 after:transition-all dark:border-gray-600 peer-checked:bg-[#303853] peer-checked:dark:bg-blue-200" />
    </label>
  )
}

export default ThemeSwitch
