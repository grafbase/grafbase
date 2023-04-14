import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import monacoEditorPlugin from 'vite-plugin-monaco-editor'

export default defineConfig(({ command }) => {
  return {
    base:
      command === 'serve' ? '/' : 'https://assets.grafbase.com/cli/pathfinder',
    build: {
      rollupOptions: {
        output: {
          entryFileNames: `assets/[name].js`,
          chunkFileNames: `assets/[name].js`,
          assetFileNames: `assets/[name].[ext]`
        }
      }
    },
    plugins: [
      react(),
      monacoEditorPlugin({
        customDistPath: (_root, buildOutDir) => {
          // this ensures that our workers will be copied to the default folder (monacoeditorwork) next to /assets in the build dir
          return buildOutDir + '/' + 'monacoeditorwork'
        },
        languageWorkers: ['json', 'editorWorkerService'],
        customWorkers: [
          {
            label: 'graphql',
            entry: 'monaco-graphql/dist/graphql.worker'
          }
        ]
      })
    ]
  }
})
