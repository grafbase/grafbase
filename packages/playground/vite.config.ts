import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import dts from 'vite-plugin-dts'

export default defineConfig({
  base: './',
  define: {
    'process.env': process.env
  },
  build: {
    minify: true,
    lib: {
      entry: 'src/index.ts',
      formats: ['es']
    },
    rollupOptions: {
      external: ['react', 'react-dom', 'graphql'],
      output: {
        globals: {
          react: 'React',
          'react-dom': 'ReactDOM',
          graphql: 'GraphQL'
        }
      }
    }
  },
  plugins: [react(), dts()]
})
