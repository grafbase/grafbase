import {
  PlayIcon,
  StopIcon,
  ToolbarButton,
  useExecutionContext
} from '@graphiql/react'
import { SSEStatus, useSSEContext } from '../utils/sse'

const ExecuteButton = () => {
  const { isFetching, run, stop } = useExecutionContext({ nonNull: true })
  const stream = useSSEContext()

  const fetching = isFetching || stream.status === SSEStatus.OPEN
  const label = `${fetching ? 'Stop' : 'Execute'} query (Ctrl-Enter)`

  const onClick = () => {
    if (fetching) {
      stop()
    } else {
      run()
    }
  }

  if (stream.status !== SSEStatus.CLOSED) {
    return (
      <ToolbarButton
        type="button"
        label={label}
        aria-label={label}
        className="graphiql-execute-button"
        style={{ backgroundColor: 'hsl(var(--color-error))' }}
        onClick={onClick}
      >
        {fetching ? <StopIcon /> : <PlayIcon />}
      </ToolbarButton>
    )
  }

  return (
    <ToolbarButton
      type="button"
      disabled={fetching}
      label={label}
      aria-label={label}
      className="graphiql-execute-button"
      style={fetching ? { opacity: 0.7, cursor: 'not-allowed' } : undefined}
      onClick={onClick}
    >
      <PlayIcon />
    </ToolbarButton>
  )
}

export default ExecuteButton
