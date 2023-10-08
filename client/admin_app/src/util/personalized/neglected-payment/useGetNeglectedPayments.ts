import { ref } from 'vue'
import { getNeglectedPayments } from './GetNeglectedPayments'

export function useGetNeglectedPayments () {
  const getNeglectedPaymentsDone = ref(true)
  const getNeglectedPaymentsFunc = async (page: number, perPage: number) => {
    try {
      getNeglectedPaymentsDone.value = false
      const response = await getNeglectedPayments(page, perPage)
      return response
    } finally {
      getNeglectedPaymentsDone.value = true
    }
  }
  return {
    getNeglectedPaymentsDone,
    getNeglectedPaymentsFunc
  }
}
