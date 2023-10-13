import { Context, Info } from '@grafbase/sdk'

export default async function(parent, _, { kv }: Context, info: Info) {
    const { value } = await kv.get(`answers/${parent.id}`)
    return `⚙️ Result of ${info.fieldName}: ${value}`
}
