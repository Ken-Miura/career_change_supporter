import { ref } from 'vue'
import { getRefundByConsultationId } from './GetRefundByConsultationId'

export function useRefundByConsultationId () {
  const getRefundByConsultationIdDone = ref(true)
  const getRefundByConsultationIdFunc = async (consultationId: string) => {
    try {
      getRefundByConsultationIdDone.value = false
      const response = await getRefundByConsultationId(consultationId)
      return response
    } finally {
      getRefundByConsultationIdDone.value = true
    }
  }
  return {
    getRefundByConsultationIdDone,
    getRefundByConsultationIdFunc
  }
}
