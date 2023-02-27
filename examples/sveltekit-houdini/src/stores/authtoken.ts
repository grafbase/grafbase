import { browser } from '$app/environment';
import { writable } from 'svelte/store';

const initialValue = browser ? window.localStorage.getItem('authToken') : '';

export const authToken = writable<string>(initialValue!);

authToken.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem('authToken', value);
	}
});
