import { ref } from 'vue'
import { getReceiptByConsultationId } from './GetReceiptByConsultationId'

export function useReceiptByConsultationId () {
  const getReceiptByConsultationIdDone = ref(true)
  const getReceiptByConsultationIdFunc = async (consultationId: string) => {
    try {
      getReceiptByConsultationIdDone.value = false
      const response = await getReceiptByConsultationId(consultationId)
      return response
    } finally {
      getReceiptByConsultationIdDone.value = true
    }
  }
  return {
    getReceiptByConsultationIdDone,
    getReceiptByConsultationIdFunc
  }
}
