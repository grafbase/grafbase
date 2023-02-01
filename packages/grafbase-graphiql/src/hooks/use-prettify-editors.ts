import { useEditorContext } from '@graphiql/react'
import { useCallback } from 'react'
import { prettify } from '../utils/prettify'

export const usePrettifyEditors = ({
  caller
}: {
  caller?: Function
} = {}) => {
  const { queryEditor, headerEditor, variableEditor } = useEditorContext({
    nonNull: true,
    caller: caller || usePrettifyEditors
  })
  return useCallback(async () => {
    if (variableEditor) {
      const variableEditorContent = variableEditor.getValue()
      try {
        const prettifiedVariableEditorContent = JSON.stringify(
          JSON.parse(variableEditorContent),
          null,
          2
        )
        if (prettifiedVariableEditorContent !== variableEditorContent) {
          variableEditor.setValue(prettifiedVariableEditorContent)
        }
      } catch {
        /* Parsing JSON failed, skip prettification */
      }
    }

    if (headerEditor) {
      const headerEditorContent = headerEditor.getValue()

      try {
        const prettifiedHeaderEditorContent = JSON.stringify(
          JSON.parse(headerEditorContent),
          null,
          2
        )
        if (prettifiedHeaderEditorContent !== headerEditorContent) {
          headerEditor.setValue(prettifiedHeaderEditorContent)
        }
      } catch {
        /* Parsing JSON failed, skip prettification */
      }
    }

    if (queryEditor) {
      try {
        const editorContent = queryEditor.getValue()
        const prettifiedEditorContent = await prettify(editorContent)
        if (prettifiedEditorContent !== editorContent) {
          queryEditor.setValue(prettifiedEditorContent)
        }
      } catch {
        /* Parsing failed, skip prettification */
      }
    }
  }, [queryEditor, variableEditor, headerEditor])
}
