/// <references types="houdini-svelte">

/** @type {import('houdini').ConfigFile} */
const config = {
	apiUrl: function (env) {
		return env.GRAFBASE_API_URL;
	},
	schemaPollHeaders: {
		Authentication: function (env) {
			return env.GRAFBASE_API_KEY;
		}
	},
	plugins: {
		'houdini-svelte': {}
	},
	scalars: {
		DateTime: {
			type: 'DateTime',
			unmarshal(val) {
				return new Date(val);
			},
			marshal(date) {
				return date.getTime;
			}
		}
	}
};

export default config;
