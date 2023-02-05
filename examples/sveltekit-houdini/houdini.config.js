/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	plugins: {
		'houdini-svelte': {},
		'@grafbase/houdini': {}
	},
	watchSchema: {
		url: 'env:PUBLIC_API_URL',
		headers: {
			'x-api-key': 'env:PUBLIC_API_KEY'
		}
	}
};

export default config;
