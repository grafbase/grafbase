import { defineNuxtConfig } from 'nuxt'
import UnpluginComponentsVite from 'unplugin-vue-components/vite'
import IconsResolver from 'unplugin-icons/resolver'

// https://v3.nuxtjs.org/api/configuration/nuxt.config
export default defineNuxtConfig({
  srcDir: 'src',

  build: {
    transpile: ['@headlessui/vue']
  },

  buildModules: ['@unocss/nuxt', 'unplugin-icons/nuxt', 'nuxt-graphql-codegen'],

  modules: ['@nuxtjs/color-mode'],

  publicRuntimeConfig: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    githubClientId: process.env.GITHUB_CLIENT_ID
  },

  vite: {
    plugins: [
      UnpluginComponentsVite({
        dts: true,
        resolvers: [
          IconsResolver({
            prefix: 'Icon'
          })
        ]
      })
    ]
  },

  colorMode: {
    classSuffix: ''
  },

  unocss: {
    preflight: true,
    icons: true
  },

  typescript: {
    strict: true
  }
})
