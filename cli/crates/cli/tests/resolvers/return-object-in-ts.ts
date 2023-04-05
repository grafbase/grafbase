export default function Resolver({ parent, args, context, info }) {
    const returnValue: any = { a: 123, b: "Hello" };
    return returnValue;
}
