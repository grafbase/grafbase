import { errorExchange as urqlErrorExchange } from '@urql/svelte';

export const errorExchange = () =>
	urqlErrorExchange({
		onError(error) {
			if (error.graphQLErrors[0]) {
				const message = error.graphQLErrors[0].message;
				// @todo error
			} else {
				// @todo network error
			}
		}
	});
