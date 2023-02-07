import { Source, parse } from 'graphql'

export const validateQuery = (source?: string | Source | null) => {
  try {
    if (source) {
      return !!parse(source)
    }
  } catch {}

  return false
}
