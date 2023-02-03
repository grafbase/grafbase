import { useEditorContext, useSchemaContext } from '@graphiql/react'
import { mergeAst } from '@graphiql/toolkit'
import { useCallback } from 'react'
import { prettifyConfig } from '../utils/prettify'

export const useMergeQuery = ({ caller }: { caller?: Function } = {}) => {
  const { queryEditor } = useEditorContext({
    nonNull: true,
    caller: caller || useMergeQuery
  })
  const { schema } = useSchemaContext({ nonNull: true, caller: useMergeQuery })
  return useCallback(async () => {
    const query = queryEditor?.getValue()
    if (!query) {
      return
    }
    const { parse, print } = await import('../utils/graphql-modular')
    const astNode = parse(query)
    // @ts-ignore - conflict between graphiql-toolkit and graphql-modular types
    const mergedQuery = mergeAst(astNode, schema)
    // @ts-ignore - conflict between graphiql-toolkit and graphql-modular types
    const prettifiedMergedQuery = print(mergedQuery, prettifyConfig)
    queryEditor.setValue(prettifiedMergedQuery)
  }, [queryEditor, schema])
}
