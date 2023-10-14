import { ref } from 'vue'
import { getRefundedPaymentByConsultationId } from './GetRefundedPaymentByConsultationId'

export function useGetRefundedPaymentByConsultationId () {
  const getRefundedPaymentByConsultationIdDone = ref(true)
  const getRefundedPaymentByConsultationIdFunc = async (consultationId: string) => {
    try {
      getRefundedPaymentByConsultationIdDone.value = false
      const response = await getRefundedPaymentByConsultationId(consultationId)
      return response
    } finally {
      getRefundedPaymentByConsultationIdDone.value = true
    }
  }
  return {
    getRefundedPaymentByConsultationIdDone,
    getRefundedPaymentByConsultationIdFunc
  }
}
