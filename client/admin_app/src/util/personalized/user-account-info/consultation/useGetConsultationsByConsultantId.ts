import { ref } from 'vue'
import { getConsultationsByConsultantId } from './GetConsultationsByConsultantId'

export function useGetConsultationsByConsultantId () {
  const getConsultationsByConsultantIdDone = ref(true)
  const getConsultationsByConsultantIdFunc = async (consultantId: string) => {
    try {
      getConsultationsByConsultantIdDone.value = false
      const response = await getConsultationsByConsultantId(consultantId)
      return response
    } finally {
      getConsultationsByConsultantIdDone.value = true
    }
  }
  return {
    getConsultationsByConsultantIdDone,
    getConsultationsByConsultantIdFunc
  }
}
