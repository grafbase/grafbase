/**
 * Throws, if given input value is a valid GraphQL and JavaScript identifier.
 */
export function validateIdentifier(identifier: string) {
  const identifierRE = new RegExp(/^[_a-zA-Z][_a-zA-Z0-9]*$/)

  if (!identifierRE.test(identifier)) {
    throw `Given name "${identifier}" is not a valid TypeScript identifier.`
  }
}