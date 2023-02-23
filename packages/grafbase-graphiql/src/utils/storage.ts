import { Storage } from '@graphiql/toolkit'

const storages: any = {}

// Custom storage for GraphiQL state in localStorage
const makeStorage = (storageKey: string) => {
  return {
    setItem: (key: string, val: any) =>
      window.localStorage.setItem(`${storageKey}:${key}`, val),
    getItem: (key: string) =>
      window.localStorage.getItem(`${storageKey}:${key}`),
    removeItem: (key: string) =>
      window.localStorage.removeItem(`${storageKey}:${key}`)
  }
}

export const getStorage = (storageKey: string): Storage => {
  if (!storages[storageKey]) {
    storages[storageKey] = makeStorage(storageKey)
  }
  return storages[storageKey]
}
