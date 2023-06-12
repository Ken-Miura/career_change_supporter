import { ref } from 'vue'
import { getSettlementByConsultationId } from './GetSettlementByConsultationId'

export function useSettlementByConsultationId () {
  const getSettlementByConsultationIdDone = ref(true)
  const getSettlementByConsultationIdFunc = async (consultationId: string) => {
    try {
      getSettlementByConsultationIdDone.value = false
      const response = await getSettlementByConsultationId(consultationId)
      return response
    } finally {
      getSettlementByConsultationIdDone.value = true
    }
  }
  return {
    getSettlementByConsultationIdDone,
    getSettlementByConsultationIdFunc
  }
}
