import type { RequestHandler } from './__types';
import dotenv from 'dotenv';

dotenv.config();

export const POST: RequestHandler = async ({ request }) => {
	const response = await fetch(process.env.GRAFBASE_API_URL as string, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${process.env.GRAFBASE_API_KEY}`
		},
		body: JSON.stringify(await request.json())
	}).then((response) => response.json());

	return {
		body: response
	};
};

// If the user has JavaScript disabled, the URL will change to
// include the method override unless we redirect back to /todos
const redirect = {
	status: 303,
	headers: {
		location: '/'
	}
};
