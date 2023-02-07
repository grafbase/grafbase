import { EditorContextType } from '@graphiql/react'
import { useEffect } from 'react'

type EmptyCallback = () => void
type CodeMirrorEditor = EditorContextType['variableEditor']

export const useKeyMap = (
  editor: CodeMirrorEditor | null,
  keys: string[],
  callback: EmptyCallback | undefined
) => {
  useEffect(() => {
    if (!editor) {
      return
    }
    for (const key of keys) {
      editor.removeKeyMap(key)
    }

    if (callback) {
      const keyMap: Record<string, EmptyCallback> = {}
      for (const key of keys) {
        keyMap[key] = () => callback()
      }
      editor.addKeyMap(keyMap)
    }
  }, [editor, keys, callback])
}
