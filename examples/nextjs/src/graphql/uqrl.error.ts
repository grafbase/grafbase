import { errorExchange as urqlErrorExchange } from 'urql'
import { toast } from 'react-toastify'

export const errorExchange = () =>
  urqlErrorExchange({
    onError(error) {
      if (error.graphQLErrors[0]) {
        const message = error.graphQLErrors[0].message
        toast(message, {
          type: 'error',
          className: 'dark:bg-zinc-700 dark:text-white'
        })
      } else {
        toast('Network error', {
          type: 'error',
          className: 'dark:bg-zinc-700 dark:text-white'
        })
      }
    }
  })
