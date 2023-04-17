import { Configuration, OpenAIApi } from 'openai'

export default async function Resolver(_, { prompt }) {
  const configuration = new Configuration({
    apiKey: process.env.OPENAI_API_KEY
  })

  const openai = new OpenAIApi(configuration)

  const response = await openai.createCompletion({
    model: 'text-davinci-003',
    prompt,
    max_tokens: 7,
    temperature: 0
  })

  return response.choices || []
}
