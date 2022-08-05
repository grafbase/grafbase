import { errorExchange as urqlErrorExchange } from '@urql/svelte';
import { toast } from '@zerodevx/svelte-toast';

const toastOptions = {
	theme: {
		'--toastBackground': '#F56565',
		'--toastBarBackground': '#C53030'
	}
};

export const errorExchange = () =>
	urqlErrorExchange({
		onError(error) {
			if (error.graphQLErrors[0]) {
				const message = error.graphQLErrors[0].message;
				toast.push(message, toastOptions);
			} else {
				toast.push('Network error', toastOptions);
			}
		}
	});
