import { ref } from 'vue'
import { getAwaitingPaymentByConsultationId } from './GetAwaitingPaymentByConsultationId'

export function useGetAwaitingPaymentByConsultationId () {
  const getAwaitingPaymentByConsultationIdDone = ref(true)
  const getAwaitingPaymentByConsultationIdFunc = async (consultationId: string) => {
    try {
      getAwaitingPaymentByConsultationIdDone.value = false
      const response = await getAwaitingPaymentByConsultationId(consultationId)
      return response
    } finally {
      getAwaitingPaymentByConsultationIdDone.value = true
    }
  }
  return {
    getAwaitingPaymentByConsultationIdDone,
    getAwaitingPaymentByConsultationIdFunc
  }
}
