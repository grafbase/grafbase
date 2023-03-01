import type { Handle } from '@sveltejs/kit';
import { setSession } from '$houdini';

export const handle = (async ({ event, resolve }) => {
	const authToken = event.cookies.get('authToken');

	setSession(event, { authToken });

	return await resolve(event);
}) satisfies Handle;
