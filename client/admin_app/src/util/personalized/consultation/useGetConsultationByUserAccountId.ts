import { ref } from 'vue'
import { getConsultationByConsultationId } from './GetConsultationByConsultationId'

export function useGetConsultationByConsultationId () {
  const getConsultationByConsultationIdDone = ref(true)
  const getConsultationByConsultationIdFunc = async (consultationId: string) => {
    try {
      getConsultationByConsultationIdDone.value = false
      const response = await getConsultationByConsultationId(consultationId)
      return response
    } finally {
      getConsultationByConsultationIdDone.value = true
    }
  }
  return {
    getConsultationByConsultationIdDone,
    getConsultationByConsultationIdFunc
  }
}
