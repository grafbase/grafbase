import type { Fetcher, FetcherParams } from '@graphiql/toolkit'
import { isLiveQueryOperationDefinitionNode } from '@n1ru4l/graphql-live-query'
import { applyLiveQueryJSONPatch } from '@n1ru4l/graphql-live-query-patch-json-patch'
import {
  applyAsyncIterableIteratorToSink,
  makeAsyncIterableIteratorFromSink
} from '@n1ru4l/push-pull-async-iterable-iterator'
import { Repeater } from '@repeaterjs/repeater'
import { DocumentNode, getOperationAST, GraphQLError } from 'graphql'
import { createContext, ReactNode, useContext, useState } from 'react'
import ReconnectingEventSource from 'reconnecting-eventsource'

export enum SSEStatus {
  CLOSED = 'CLOSED',
  OPEN = 'OPEN',
  CONNECTING = 'CONNECTING'
}

type FetcherOptions = {
  url: string
  headers?: Record<string, string> | undefined
}
type SSEFetcherOptions = FetcherOptions & {
  statusCallback?: (status: SSEStatus) => void
}

interface ExecutionResult<
  Data = Record<string, unknown>,
  Extensions = Record<string, unknown>
> {
  errors?: ReadonlyArray<GraphQLError>
  data?: Data | null
  hasNext?: boolean
  extensions?: Extensions
}
type SSEClient = (args: {
  url: string
  statusCallback?: (status: SSEStatus) => void
}) => Repeater<ExecutionResult, any, unknown>

export const isLiveQuery = (
  document: DocumentNode,
  name: string | undefined
): boolean => {
  const documentNode = getOperationAST(document, name)
  return !!documentNode
    ? isLiveQueryOperationDefinitionNode(documentNode)
    : false
}

const sseClient: SSEClient = ({ url, statusCallback }) => {
  return new Repeater(async (push, stop) => {
    statusCallback?.(SSEStatus.CONNECTING)
    const eventSource = new ReconnectingEventSource(url)
    eventSource.onmessage = (event) => {
      statusCallback?.(SSEStatus.OPEN)
      push(JSON.parse(event.data))
      if (eventSource.readyState === EventSource.CLOSED) {
        stop()
      }
    }
    eventSource.onerror = (error) => {
      if (error.isTrusted) {
        stop('NetworkError: browser closed the connection')
      } else {
        stop(error)
      }
    }
    await stop
    eventSource.close()
    statusCallback?.(SSEStatus.CLOSED)
  })
}

export const getSseFetcher = ({
  statusCallback,
  ...options
}: SSEFetcherOptions) => {
  return (graphQLParams: FetcherParams) => {
    const headers = Object.entries(options.headers || {}).reduce(
      (accumulator, [key, value]) => {
        if (value === undefined) {
          return accumulator
        }
        return {
          ...accumulator,
          [key]: value
        }
      },
      {} as { [key: string]: any }
    )
    const searchParams = new URLSearchParams({
      ...headers,
      query: graphQLParams.query,
      operationName: graphQLParams.operationName || '',
      variables: JSON.stringify(graphQLParams.variables || {})
    })
    const url = `${options.url}?${searchParams.toString()}`
    const client = sseClient({ url, statusCallback })

    return makeAsyncIterableIteratorFromSink<ExecutionResult>((sink) =>
      applyAsyncIterableIteratorToSink(applyLiveQueryJSONPatch(client), sink)
    )
  }
}

type SSEFetcherType = (options: FetcherOptions) => Fetcher

type SSEContextType = {
  sseFetcher: SSEFetcherType
  status: SSEStatus
}

const SSEContext = createContext<SSEContextType | null>(null)
SSEContext.displayName = 'SSEContext'

export const SSEProvider = ({ children }: { children?: ReactNode }) => {
  const [status, setStatus] = useState<SSEStatus>(SSEStatus.CLOSED)

  const sseFetcher: SSEFetcherType = ({ url, headers }) =>
    getSseFetcher({ url, headers, statusCallback: setStatus })

  return (
    <SSEContext.Provider value={{ sseFetcher, status }}>
      {children}
    </SSEContext.Provider>
  )
}

export const useSSEContext = () => {
  const context = useContext(SSEContext)
  if (!context) {
    throw new Error('SSEContext not found')
  }
  return context
}
