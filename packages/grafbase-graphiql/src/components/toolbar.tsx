import {
  CopyIcon,
  MergeIcon,
  PrettifyIcon,
  ToolbarButton,
  useCopyQuery,
  useEditorContext
} from '@graphiql/react'
import { useKeyMap } from '../hooks/use-key-map'
import { useMergeQuery } from '../hooks/use-merge-query'
import { usePrettifyEditors } from '../hooks/use-prettify-editors'
import ExecuteButton from './execute-button'

export const Toolbar = () => {
  const { queryEditor, variableEditor } = useEditorContext({ nonNull: true })
  const copy = useCopyQuery()
  const merge = useMergeQuery()
  const prettify = usePrettifyEditors()

  // Replace default prettier hotkey
  useKeyMap(queryEditor, ['Shift-Ctrl-P'], prettify)
  useKeyMap(variableEditor, ['Shift-Ctrl-P'], prettify)

  return (
    <>
      <ExecuteButton />
      <ToolbarButton onClick={prettify} label="Prettify query (Shift-Ctrl-P)">
        <PrettifyIcon className="graphiql-toolbar-icon" aria-hidden="true" />
      </ToolbarButton>
      <ToolbarButton
        onClick={merge}
        label="Merge fragments into query (Shift-Ctrl-M)"
      >
        <MergeIcon className="graphiql-toolbar-icon" aria-hidden="true" />
      </ToolbarButton>
      <ToolbarButton onClick={copy} label="Copy query (Shift-Ctrl-C)">
        <CopyIcon className="graphiql-toolbar-icon" aria-hidden="true" />
      </ToolbarButton>
    </>
  )
}
