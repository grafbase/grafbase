import { Storage } from '@graphiql/toolkit'

const storages: any = {}

// Custom storage for GraphiQL state in localStorage
const makeStorage = (storageKey: string) => {
  return {
    setItem: (key: string, val: any) => window.localStorage.setItem(`${storageKey}:${key}`, val),
    getItem: (key: string) => window.localStorage.getItem(`${storageKey}:${key}`),
    removeItem: (key: string) => window.localStorage.removeItem(`${storageKey}:${key}`)
  }
}

export const getStorage = (storageKey: string): Storage => {
  if (!storages[storageKey]) {
    storages[storageKey] = makeStorage(storageKey)
  }
  return storages[storageKey]
}

const QUERY_KEY = `graphiql:query`

export const setPlaygroundQuery = (key: string, value: string) => {
  if (typeof window === 'undefined') {
    return null
  }
  return localStorage.setItem(`${key}:${QUERY_KEY}`, value)
}

const TABS_KEY = `graphiql:tabState`

export const getPlaygroundTabs = (key: string): string | null => {
  if (typeof window === 'undefined') {
    return null
  }
  return localStorage.getItem(`${key}:${TABS_KEY}`)
}

export const setPlaygroundTabs = (key: string, value: string) => {
  if (typeof window === 'undefined') {
    return null
  }
  return localStorage.setItem(`${key}:${TABS_KEY}`, value)
}

export const removePlaygroundTabs = (key: string) => {
  if (typeof window === 'undefined') {
    return null
  }
  return localStorage.removeItem(`${key}:${TABS_KEY}`)
}
