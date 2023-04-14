/// <reference types="vite/client" />

// vite
interface ImportMetaEnv {
  readonly VITE_GRAFBASE_API_KEY: string
  readonly VITE_GRAFBASE_ENDPOINT: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

// browser

// we add a placeholder string for GRAPHQL_URL on window in index.html
// when run from the cli, this string is replaced with a valid grafbase endpoint url
declare global {
  interface Window {
    GRAPHQL_URL: string
  }
}

// typescript 🤷‍♂️
export {}
