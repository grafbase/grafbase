export const prettify = async (source: string) => {
  const { parse, print } = await import('./graphql-modular')

  return print(parse(source), prettifyConfig)
}

export const prettifyConfig = {
  preserveComments: true,
  pretty: true
}
