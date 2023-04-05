export default function Resolver({ parent, args, context, info }) {
    return process.env[args.name] || null;
}
