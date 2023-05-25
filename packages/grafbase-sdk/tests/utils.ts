import { parse } from 'graphql'

export function renderGraphQL(obj: any): string {
  const stringified = obj.toString()
  try { 
    // check if it's valid graphql
    parse(stringified)

    return stringified
  } catch (e: any) {
    console.log(stringified)
    throw e
  }
}