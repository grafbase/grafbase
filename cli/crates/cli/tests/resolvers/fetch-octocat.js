export default async function Resolver({ parent, args, context, info }) {
    let response = await fetch('https://api.github.com/octocat?s=' + encodeURIComponent(args.text), {
        headers: {
            'user-agent': 'my-github-resolver'
        }
    });
    return await response.text();
}
