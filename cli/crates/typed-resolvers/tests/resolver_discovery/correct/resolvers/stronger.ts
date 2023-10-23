import { User, Resolver } from '@grafbase/generated'

export const resolver: Resolver["User.stronger"] = async (_: User, { otherUser: __ }) => {
    return true;// you're the strongest
}

export default resolver
