const serverSideFetch = async (query: string) =>
  await fetch(process.env.GRAFBASE_API_URL!, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': process.env.GRAFBASE_API_KEY!
    },
    body: JSON.stringify(query)
  }).then((response) => response.json())

export default serverSideFetch
