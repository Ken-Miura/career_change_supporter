import { ref } from 'vue'
import { getNeglectedPaymentByConsultationId } from './GetNeglectedPaymentByConsultationId'

export function useGetNeglectedPaymentByConsultationId () {
  const getNeglectedPaymentByConsultationIdDone = ref(true)
  const getNeglectedPaymentByConsultationIdFunc = async (consultationId: string) => {
    try {
      getNeglectedPaymentByConsultationIdDone.value = false
      const response = await getNeglectedPaymentByConsultationId(consultationId)
      return response
    } finally {
      getNeglectedPaymentByConsultationIdDone.value = true
    }
  }
  return {
    getNeglectedPaymentByConsultationIdDone,
    getNeglectedPaymentByConsultationIdFunc
  }
}
