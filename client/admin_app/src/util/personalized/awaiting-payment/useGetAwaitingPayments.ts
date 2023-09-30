import { ref } from 'vue'
import { getAwaitingPayments } from './GetAwaitingPayments'

export function useGetAwaitingPayments () {
  const getAwaitingPaymentsDone = ref(true)
  const getAwaitingPaymentsFunc = async (page: number, perPage: number) => {
    try {
      getAwaitingPaymentsDone.value = false
      const response = await getAwaitingPayments(page, perPage)
      return response
    } finally {
      getAwaitingPaymentsDone.value = true
    }
  }
  return {
    getAwaitingPaymentsDone,
    getAwaitingPaymentsFunc
  }
}
