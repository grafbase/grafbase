import type { Actions } from './$types';

import { SignJWT } from 'jose';

const issuerUrl = 'https://grafbase.com';

const secret = new Uint8Array('abc'.split('').map((c) => c.charCodeAt(0)));

const getToken = (role: string) => {
	const groups = role ? [role] : [];
	return new SignJWT({ sub: 'user_1234', groups })
		.setProtectedHeader({ alg: 'HS256', typ: 'JWT' })
		.setIssuer(issuerUrl)
		.setIssuedAt()
		.setExpirationTime('2h')
		.sign(secret);
};

export const actions = {
	default: async ({ cookies, request }) => {
		const data = await request.formData();
		const role = data.get('role');

		if (role) {
			cookies.set('authToken', await getToken(role as string));
		}

		return { success: true };
	}
} satisfies Actions;
