import { sveltekit } from '@sveltejs/kit/vite'
import path from 'path'

/** @type {import('vite').UserConfig} */
const config = {
  plugins: [sveltekit()],
  resolve: {
    alias: {
      $graphql: path.resolve(__dirname, 'src', 'graphql'),
      $lib: path.resolve(__dirname, 'src', 'lib')
    }
  },
  optimizeDeps: {
    exclude: ['@urql/svelte']
  }
}

export default config
