import { ref } from 'vue'
import { getRefundedPayments } from './GetRefundedPayments'

export function useGetRefundedPayments () {
  const getRefundedPaymentsDone = ref(true)
  const getRefundedPaymentsFunc = async (page: number, perPage: number) => {
    try {
      getRefundedPaymentsDone.value = false
      const response = await getRefundedPayments(page, perPage)
      return response
    } finally {
      getRefundedPaymentsDone.value = true
    }
  }
  return {
    getRefundedPaymentsDone,
    getRefundedPaymentsFunc
  }
}
