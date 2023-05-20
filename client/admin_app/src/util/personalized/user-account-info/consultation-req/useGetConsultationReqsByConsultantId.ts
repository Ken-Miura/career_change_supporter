import { ref } from 'vue'
import { getConsultationReqsByConsultantId } from './GetConsultationReqsByConsultantId'

export function useGetConsultationReqsByConsultantId () {
  const getConsultationReqsByConsultantIdDone = ref(true)
  const getConsultationReqsByConsultantIdFunc = async (consultantId: string) => {
    try {
      getConsultationReqsByConsultantIdDone.value = false
      const response = await getConsultationReqsByConsultantId(consultantId)
      return response
    } finally {
      getConsultationReqsByConsultantIdDone.value = true
    }
  }
  return {
    getConsultationReqsByConsultantIdDone,
    getConsultationReqsByConsultantIdFunc
  }
}
