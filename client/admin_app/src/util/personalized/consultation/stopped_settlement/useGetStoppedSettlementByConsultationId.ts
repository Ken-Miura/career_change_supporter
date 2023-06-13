import { ref } from 'vue'
import { getStoppedSettlementByConsultationId } from './GetStoppedSettlementByConsultationId'

export function useStoppedSettlementByConsultationId () {
  const getStoppedSettlementByConsultationIdDone = ref(true)
  const getStoppedSettlementByConsultationIdFunc = async (consultationId: string) => {
    try {
      getStoppedSettlementByConsultationIdDone.value = false
      const response = await getStoppedSettlementByConsultationId(consultationId)
      return response
    } finally {
      getStoppedSettlementByConsultationIdDone.value = true
    }
  }
  return {
    getStoppedSettlementByConsultationIdDone,
    getStoppedSettlementByConsultationIdFunc
  }
}
