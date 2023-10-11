import { ref } from 'vue'
import { getLeftAwaitingWithdrawals } from './GetLeftAwaitingWithdrawals'

export function useGetRefundedPayments () {
  const getLeftAwaitingWithdrawalsDone = ref(true)
  const getLeftAwaitingWithdrawalsFunc = async (page: number, perPage: number) => {
    try {
      getLeftAwaitingWithdrawalsDone.value = false
      const response = await getLeftAwaitingWithdrawals(page, perPage)
      return response
    } finally {
      getLeftAwaitingWithdrawalsDone.value = true
    }
  }
  return {
    getLeftAwaitingWithdrawalsDone,
    getLeftAwaitingWithdrawalsFunc
  }
}
