import type { Actions } from './$types';
import { fail } from '@sveltejs/kit';
import { SignJWT } from 'jose';
import { graphql } from '$houdini';
import { GRAFBASE_ISSUER_URL, GRAFBASE_JWT_SECRET } from '$env/static/private';

const secret = new Uint8Array(GRAFBASE_JWT_SECRET.split('').map((c: any) => c.charCodeAt(0)));

const getToken = (role: string) => {
	const groups = role ? [role] : [];
	return new SignJWT({ sub: 'user_1234', groups })
		.setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
		.setIssuer(GRAFBASE_ISSUER_URL)
		.setIssuedAt()
		.setExpirationTime('2h')
		.sign(secret);
};

export const actions = {
	auth: async ({ cookies, request }) => {
		const data = await request.formData();
		const role = data.get('role');

		if (role) {
			cookies.set('authToken', await getToken(role as string));
		}

		return { success: true };
	},
	add: async (event) => {
		const data = await event.request.formData();

		const author = data.get('author')?.toString();
		const message = data.get('message')?.toString();

		if (!author) {
			return fail(403, { author: '*' });
		}

		if (!message) {
			return fail(403, { message: '*' });
		}

		const addMessage = graphql(`
			mutation addMessage($author: String!, $message: String!) {
				messageCreate(input: { author: $author, message: $message }) {
					message {
						id
						author
						message
						createdAt
					}
				}
			}
		`);

		return await addMessage.mutate({ author, message }, { event });
	}
} satisfies Actions;
