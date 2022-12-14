export const grafbaseClient = async <T>(body: {
	query: string;
	variables: Record<string, any>;
}): Promise<T> => {
    const response = await fetch(import.meta.env.VITE_GRAFBASE_API_URL as string, {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          'x-api-key': import.meta.env.VITE_GRAFBASE_API_KEY as string
        },
        body: JSON.stringify(body)
      })
	const json: any = await response.json();
	return json.data;
};
