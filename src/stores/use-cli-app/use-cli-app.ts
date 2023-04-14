import { create } from 'zustand'

import { UseCliAppStore } from './use-cli-app.types'

export const useCliApp = create<UseCliAppStore>((set, get) => ({
  theme: window.matchMedia('(prefers-color-scheme:dark)').matches
    ? 'dark'
    : 'light',
  toggleTheme: () => {
    const theme = get().theme

    if (theme === 'dark') {
      return set({ theme: 'light' })
    }

    if (theme === 'light') {
      return set({ theme: 'dark' })
    }
  },
  visibleTool: 'Pathfinder',
  setVisibleTool: ({ visibleTool }) => {
    set({ visibleTool })
  }
}))
