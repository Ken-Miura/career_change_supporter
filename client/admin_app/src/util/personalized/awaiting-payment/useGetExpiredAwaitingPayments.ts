import { ref } from 'vue'
import { getExpiredAwaitingPayments } from './GetExpiredAwaitingPayments'

export function useGetExpiredAwaitingPayments () {
  const getExpiredAwaitingPaymentsDone = ref(true)
  const getExpiredAwaitingPaymentsFunc = async (page: number, perPage: number) => {
    try {
      getExpiredAwaitingPaymentsDone.value = false
      const response = await getExpiredAwaitingPayments(page, perPage)
      return response
    } finally {
      getExpiredAwaitingPaymentsDone.value = true
    }
  }
  return {
    getExpiredAwaitingPaymentsDone,
    getExpiredAwaitingPaymentsFunc
  }
}
