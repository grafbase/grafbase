export default function Resolver({ parent, args, context, info }) {
    return fetch('https://api.grafbase.com/graphql', {
        headers: {
            'content-type': 'application/json'
        },
        method: 'POST',
        body: JSON.stringify({ query: '{ __typename }' })
    });
}
