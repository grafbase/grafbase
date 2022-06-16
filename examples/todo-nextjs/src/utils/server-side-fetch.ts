const serverSideFetch = async (query : string) => await fetch(process.env.GRAFBASE_API_ENDPOINT as string, {
    method: "POST",
    headers: {
        "Content-Type": "application/json",
        "Authorization" : `Bearer ${process.env.GRAFBASE_API_KEY}`
    },
    body: JSON.stringify(
        query
    )
}).then((data) => data.json())

export default serverSideFetch