import { ref } from 'vue'
import { getConsultationRequestDetail } from './GetConsultationRequestDetail'

export function usegetConsultationRequestDetail (consultationRequestId: string) {
  const getConsultationRequestDetailDone = ref(true)
  const getConsultationRequestDetailFunc = async () => {
    try {
      getConsultationRequestDetailDone.value = false
      const response = await getConsultationRequestDetail(consultationRequestId)
      return response
    } finally {
      getConsultationRequestDetailDone.value = true
    }
  }
  return {
    getConsultationRequestDetailDone,
    getConsultationRequestDetailFunc
  }
}
