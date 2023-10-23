import { Resolver } from '@grafbase/generated'

const resolver: Resolver['Question.getAnswer'] = async (parent, args, { kv }, info) => {
    const { value } = await kv.get(`answers/${parent.id}`)
    return `⚙️ Result of ${info.fieldName}: ${value}`
}

export default resolver
