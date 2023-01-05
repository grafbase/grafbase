import { graphql, type GetAllMessages$input } from '$houdini';

export const _houdini_load = graphql(`
	query GetAllMessages($first: Int) {
		messageCollection(first: $first) {
			edges {
				node {
					id
					body
					author
					createdAt
				}
			}
		}
	}
`);

export function _GetAllMessagesVariables({ params }): GetAllMessages$input {
	return {
		first: params?.first
	};
}
