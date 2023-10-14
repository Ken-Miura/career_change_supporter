import { ref } from 'vue'
import { getReceiptOfConsultationByConsultationId } from './GetReceiptOfConsultationByConsultationId'

export function useGetReceiptOfConsultationByConsultationId () {
  const getReceiptOfConsultationByConsultationIdDone = ref(true)
  const getReceiptOfConsultationByConsultationIdFunc = async (consultationId: string) => {
    try {
      getReceiptOfConsultationByConsultationIdDone.value = false
      const response = await getReceiptOfConsultationByConsultationId(consultationId)
      return response
    } finally {
      getReceiptOfConsultationByConsultationIdDone.value = true
    }
  }
  return {
    getReceiptOfConsultationByConsultationIdDone,
    getReceiptOfConsultationByConsultationIdFunc
  }
}
