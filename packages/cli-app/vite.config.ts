import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig(({ command }) => {
  return {
    build: {
      rollupOptions: {
        output: {
          // this prevents vite from hashing assets (produces index.js rather than index-XXXXXXXX.js)
          entryFileNames: `assets/[name].js`,
          chunkFileNames: `assets/[name].js`,
          assetFileNames: `assets/[name].[ext]`
        }
      }
    },
    plugins: [react()],
    optimizeDeps: {
      include: ['react', 'react-dom'],
      exclude: ['@pathfinder-ide/react']
    }
  }
})
