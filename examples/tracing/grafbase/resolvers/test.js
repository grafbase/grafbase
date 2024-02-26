export default async function Resolver(_, { prompt }, { ai }) {
    console.log(`prompt: ${prompt}`);

    const { response } = await ai.textLlm({ prompt })

    console.log(`${response}`)

    return response
}
