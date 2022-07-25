import { api } from './_api';
import type { RequestHandler } from './__types';

export const POST: RequestHandler = async ({ request, locals }) => {
	const form = await request.formData();

	await api('POST', `todos/${locals.userid}`, {
		text: form.get('text')
	});

	return {};
};

// If the user has JavaScript disabled, the URL will change to
// include the method override unless we redirect back to /todos
const redirect = {
	status: 303,
	headers: {
		location: '/'
	}
};
