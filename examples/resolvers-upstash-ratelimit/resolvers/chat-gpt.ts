import {Ratelimit} from '@upstash/ratelimit'
import {Redis} from '@upstash/redis'

// Create a new rate-limiter, allowing 10 requests per 10 seconds
const ratelimit = new Ratelimit({
    redis: Redis.fromEnv(),
    limiter: Ratelimit.slidingWindow(10, '10 s'),
    analytics: true,
    prefix: '@upstash/ratelimit' // Optional prefix for Redis keys
})

export default async function Resolver(_, { question }) {
    const identifier = 'api' // Use a constant string or any unique identifier
    const { success } = await ratelimit.limit(identifier)
    if (!success) {
        throw new Error('Too many requests. Please try again later.')
    }
    // Execute the expensive Chat-GPT request here
    return await expensiveChatGPTRequest(question)
}

async function expensiveChatGPTRequest(question) {
    // Make the expensive Chat-GPT request here
    const response = await fetch('https://api.openai.com/v1/chat/completions', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            // You must define OPENAI_API_KEY in your .env file
            Authorization: `Bearer ${process.env.OPENAI_API_KEY}`
        },
        body: JSON.stringify({
            messages: [
                { role: 'system', content: 'You are a helpful assistant.' },
                { role: 'user', content: question }
            ]
        })
    })
    const data = await response.json()
    // Extract and return the generated response from the API
    return data.choices[0].message.content
}
