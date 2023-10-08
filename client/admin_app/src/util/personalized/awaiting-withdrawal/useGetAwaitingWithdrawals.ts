import { ref } from 'vue'
import { getAwaitingWithdrawals } from './GetAwaitingWithdrawals'

export function useGetAwaitingWithdrawals () {
  const getAwaitingWithdrawalsDone = ref(true)
  const getAwaitingWithdrawalsFunc = async (page: number, perPage: number) => {
    try {
      getAwaitingWithdrawalsDone.value = false
      const response = await getAwaitingWithdrawals(page, perPage)
      return response
    } finally {
      getAwaitingWithdrawalsDone.value = true
    }
  }
  return {
    getAwaitingWithdrawalsDone,
    getAwaitingWithdrawalsFunc
  }
}
