// import { useEditor } from 'pathfinder'
import { create } from 'zustand'

import { UseCliAppStore } from './use-cli-app.types'

export const useCliApp = create<UseCliAppStore>((set, get) => ({
  theme: window.matchMedia('(prefers-color-scheme:dark)').matches
    ? 'dark'
    : 'light',
  toggleTheme: () => {
    const theme = get().theme

    if (theme === 'dark') {
      // useEditor.getState().setEditorTheme({
      //   theme: 'light'
      // })
      return set({ theme: 'light' })
    }

    if (theme === 'light') {
      // useEditor.getState().setEditorTheme({
      //   theme: 'dark'
      // })
      return set({ theme: 'dark' })
    }
  },
  visibleTool: 'Pathfinder',
  setVisibleTool: ({ visibleTool }) => {
    set({ visibleTool })
  }
}))
