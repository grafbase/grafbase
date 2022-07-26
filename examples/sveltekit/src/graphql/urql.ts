import { createClient, dedupExchange, fetchExchange } from '@urql/svelte';
import type { ClientOptions } from '@urql/svelte';
import { errorExchange } from '$graphql/urql.error';
import { cacheExchange } from '$graphql/urql.exchange';

const urqlClientBaseConfig: ClientOptions = {
	url: '/graphql',
	requestPolicy: 'cache-and-network'
};

export const urqlClient = createClient({
	...urqlClientBaseConfig,
	exchanges: [dedupExchange, cacheExchange(), errorExchange(), fetchExchange]
});
