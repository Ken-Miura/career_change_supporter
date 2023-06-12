import { ref } from 'vue'
import { getConsultantRatingByConsultationId } from './GetConsultantRatingByConsultationId'

export function useGetConsultantRatingByConsultationId () {
  const getConsultantRatingByConsultationIdDone = ref(true)
  const getConsultantRatingByConsultationIdFunc = async (consultationId: string) => {
    try {
      getConsultantRatingByConsultationIdDone.value = false
      const response = await getConsultantRatingByConsultationId(consultationId)
      return response
    } finally {
      getConsultantRatingByConsultationIdDone.value = true
    }
  }
  return {
    getConsultantRatingByConsultationIdDone,
    getConsultantRatingByConsultationIdFunc
  }
}
