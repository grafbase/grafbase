export const grafbaseClient = ({
  query,
  variables
}: {
  query: string | string[]
  // deno-lint-ignore no-explicit-any
  variables: { [key: string]: any }
}) =>
  fetch(Deno.env.get('GRAFBASE_API_URL') as string, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-api-key': Deno.env.get('GRAFBASE_API_KEY') as string
    },
    body: JSON.stringify({
      query,
      variables
    })
  })
