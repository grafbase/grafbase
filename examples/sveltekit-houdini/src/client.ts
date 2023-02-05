import { HoudiniClient } from '$houdini';
import { PUBLIC_API_URL, PUBLIC_API_KEY } from '$env/static/public'

export default new HoudiniClient({
	url: PUBLIC_API_URL,
	fetchParams() {
		return {
			headers: {
				'x-api-key': PUBLIC_API_KEY
			}
		};
	}
});
