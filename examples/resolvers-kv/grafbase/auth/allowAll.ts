import { AuthorizerContext, VerifiedIdentity } from '@grafbase/sdk'

export default async function ({ request }: AuthorizerContext): Promise<VerifiedIdentity> {
    console.log(JSON.stringify(request))
    return {
        identity: {
            groups: ["all"]
        }
    }
}
