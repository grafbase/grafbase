import { ApolloLink, FetchResult, Observable, Operation } from '@apollo/client'
import { isLiveQueryOperationDefinitionNode } from '@n1ru4l/graphql-live-query'
import { applyLiveQueryJSONPatch } from '@n1ru4l/graphql-live-query-patch-json-patch'
import { applyAsyncIterableIteratorToSink } from '@n1ru4l/push-pull-async-iterable-iterator'
import { Repeater } from '@repeaterjs/repeater'
import { DefinitionNode, print } from 'graphql'
import ReconnectingEventSource from 'reconnecting-eventsource'

export const isLiveQuery = (
  definition?: DefinitionNode | null | undefined,
  variables?: { [key: string]: unknown }
) => !!definition && isLiveQueryOperationDefinitionNode(definition, variables)

const makeEventStreamSource = (url: string) => {
  const eventSource = new ReconnectingEventSource(url)
  return applyLiveQueryJSONPatch(
    new Repeater<FetchResult>(async (push, end) => {
      eventSource.onmessage = (event) => {
        const data = JSON.parse(event.data)
        push(data)
        if (eventSource.readyState === 2) {
          end()
        }
      }
      eventSource.onerror = (error) => {
        end(error)
      }
      await end
      eventSource.close()
    })
  )
}

type SSELinkOptions = EventSourceInit & { uri: string; headers?: HeadersInit }

export class SSELink extends ApolloLink {
  constructor(private options: SSELinkOptions) {
    super()
  }

  public request(operation: Operation): Observable<FetchResult> {
    const { headers: contextHeaders = {} } = operation.getContext();
    const headers = Object.entries(this.options.headers ?? {}).reduce(
      (headers, [key, value]) => ({ ...headers, [key]: value }),
      {} as Record<string, string>
    )
    const searchParams = new URLSearchParams({
      ...contextHeaders,
      ...headers,
      query: print(operation.query),
      operationName: operation.operationName || '',
      variables: JSON.stringify(operation.variables || {})
    })
    const url = new URL(this.options.uri)
    url.search = searchParams.toString()
    const client = makeEventStreamSource(url.toString())
    return new Observable((sink) =>
      applyAsyncIterableIteratorToSink(client, sink)
    )
  }
}
