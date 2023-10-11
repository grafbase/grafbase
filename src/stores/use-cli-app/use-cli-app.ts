import { setPathfinderTheme } from '@pathfinder-ide/react'
import { create } from 'zustand'

import { UseCliAppStore } from './use-cli-app.types'

export const useCliApp = create<UseCliAppStore>((set, get) => ({
  theme: window.matchMedia('(prefers-color-scheme:dark)').matches
    ? 'dark'
    : 'light',
  toggleTheme: () => {
    const theme = get().theme

    if (theme === 'dark') {
      setPathfinderTheme({ theme: 'light' })
      return set({ theme: 'light' })
    }

    if (theme === 'light') {
      setPathfinderTheme({ theme: 'dark' })
      return set({ theme: 'dark' })
    }
  }
}))
